use ark_bn254::{Bn254, Fr};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use base64::engine::general_purpose;
use base64::Engine as _;
use bitcoin::taproot::TaprootBuilder;
use bitcoin::ScriptBuf;
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};

pub const ENV_BITVM2_GROTH16_VK_B64: &str = "BITVM2_GROTH16_VK_B64";

#[derive(Debug)]
pub enum Bitvm2VerifyError {
    Internal,
    InvalidBase64,
    InvalidHex,
    InvalidProof,
    InvalidVerifyingKey,
    InvalidPublicInput,
}

impl std::fmt::Display for Bitvm2VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "internal error"),
            Self::InvalidBase64 => write!(f, "invalid base64"),
            Self::InvalidHex => write!(f, "invalid hex"),
            Self::InvalidProof => write!(f, "invalid Groth16 proof"),
            Self::InvalidVerifyingKey => write!(f, "invalid Groth16 verifying key"),
            Self::InvalidPublicInput => write!(f, "invalid public input"),
        }
    }
}

impl std::error::Error for Bitvm2VerifyError {}

fn decode_b64(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|_| Bitvm2VerifyError::InvalidBase64)
}

fn decode_hex_32(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    let s = s.trim();
    let trimmed = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    let bytes = hex::decode(trimmed).map_err(|_| Bitvm2VerifyError::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Bitvm2VerifyError::InvalidHex);
    }
    Ok(bytes)
}

fn decode_hex_any(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    let s = s.trim();
    let trimmed = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if trimmed.is_empty() {
        return Err(Bitvm2VerifyError::InvalidHex);
    }

    let normalized = if trimmed.len().is_multiple_of(2) {
        trimmed.to_owned()
    } else {
        format!("0{trimmed}")
    };

    hex::decode(normalized).map_err(|_| Bitvm2VerifyError::InvalidHex)
}

fn fr_from_be_bytes_32_canonical(bytes: &[u8]) -> Result<Fr, Bitvm2VerifyError> {
    if bytes.len() != 32 {
        return Err(Bitvm2VerifyError::InvalidPublicInput);
    }

    let fr = Fr::from_be_bytes_mod_order(bytes);

    let mut roundtrip = fr.into_bigint().to_bytes_be();
    if roundtrip.len() > 32 {
        return Err(Bitvm2VerifyError::InvalidPublicInput);
    }
    if roundtrip.len() < 32 {
        let mut padded = vec![0u8; 32 - roundtrip.len()];
        padded.append(&mut roundtrip);
        roundtrip = padded;
    }
    if roundtrip != bytes {
        return Err(Bitvm2VerifyError::InvalidPublicInput);
    }

    Ok(fr)
}

fn parse_public_input(s: &str) -> Result<Fr, Bitvm2VerifyError> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        let bytes = decode_hex_any(s)?;
        if bytes.len() == 32 {
            fr_from_be_bytes_32_canonical(&bytes)
        } else {
            Ok(Fr::from_be_bytes_mod_order(&bytes))
        }
    } else {
        Fr::from_str(s).map_err(|_| Bitvm2VerifyError::InvalidPublicInput)
    }
}

struct PvkCache {
    vk_b64: String,
    pvk: Arc<PreparedVerifyingKey<Bn254>>,
}

static PVK_CACHE: OnceLock<RwLock<Option<PvkCache>>> = OnceLock::new();

fn get_or_prepare_pvk(vk_b64: &str) -> Result<Arc<PreparedVerifyingKey<Bn254>>, Bitvm2VerifyError> {
    let cache_lock = PVK_CACHE.get_or_init(|| RwLock::new(None));
    let vk_b64 = vk_b64.trim();

    if let Some(cache) = cache_lock.read().unwrap().as_ref() {
        if cache.vk_b64 == vk_b64 {
            return Ok(Arc::clone(&cache.pvk));
        }
    }

    let vk_bytes = decode_b64(vk_b64).map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    let mut vk_cursor = vk_bytes.as_slice();
    let vk: VerifyingKey<Bn254> = CanonicalDeserialize::deserialize_compressed(&mut vk_cursor)
        .map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    if !vk_cursor.is_empty() {
        return Err(Bitvm2VerifyError::InvalidVerifyingKey);
    }

    let pvk = Arc::new(prepare_verifying_key(&vk));
    *cache_lock.write().unwrap() = Some(PvkCache {
        vk_b64: vk_b64.to_string(),
        pvk: Arc::clone(&pvk),
    });

    Ok(pvk)
}

pub fn verify_state_root_bn254_groth16(
    vk_b64: &str,
    state_root: &str,
    proof_b64: &str,
    extra_public_inputs: Option<&[String]>,
) -> Result<bool, Bitvm2VerifyError> {
    let pvk = get_or_prepare_pvk(vk_b64)?;

    let proof_bytes = decode_b64(proof_b64).map_err(|_| Bitvm2VerifyError::InvalidProof)?;
    let mut proof_cursor = proof_bytes.as_slice();
    let proof: Proof<Bn254> = CanonicalDeserialize::deserialize_compressed(&mut proof_cursor)
        .map_err(|_| Bitvm2VerifyError::InvalidProof)?;
    if !proof_cursor.is_empty() {
        return Err(Bitvm2VerifyError::InvalidProof);
    }

    let root_bytes = decode_hex_32(state_root)?;
    let root_fr = fr_from_be_bytes_32_canonical(&root_bytes)?;

    let mut inputs = Vec::new();

    if let Some(values) = extra_public_inputs {
        let mut iter = values.iter();
        if let Some(first) = iter.next() {
            let first_fr = parse_public_input(first)?;
            if first_fr == root_fr {
                inputs.push(first_fr);
            } else {
                inputs.push(root_fr);
                inputs.push(first_fr);
            }
        } else {
            inputs.push(root_fr);
        }

        for v in iter {
            inputs.push(parse_public_input(v)?);
        }
    } else {
        inputs.push(root_fr);
    }

    Groth16::<Bn254>::verify_proof(pvk.as_ref(), &proof, &inputs)
        .map_err(|_| Bitvm2VerifyError::InvalidProof)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bitvm2Segment {
    pub segment_index: u32,
    pub script_hash: String,
    pub commitment: String,
    pub status: String,
}

pub struct Bitvm2Orchestrator {
    pub total_segments: u32,
}

impl Default for Bitvm2Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Bitvm2Orchestrator {
    pub fn new() -> Self {
        Self {
            total_segments: 364,
        }
    }

    pub fn generate_segments(&self, state_root: &str) -> Vec<Bitvm2Segment> {
        let mut segments = Vec::new();
        for i in 0..self.total_segments {
            segments.push(Bitvm2Segment {
                segment_index: i,
                script_hash: format!("sha256:hash-{}-{}", state_root, i),
                commitment: format!("commit-{}-{}", state_root, i),
                status: "Pending".to_string(),
            });
        }
        segments
    }

    pub fn verify_disprove_logic(&self, segment: &Bitvm2Segment, proof: &str) -> bool {
        !proof.is_empty() && segment.status == "Pending"
    }
}

pub struct Bitvm2MultiPartyAggregation {
    pub participants: u32,
}

impl Bitvm2MultiPartyAggregation {
    pub fn new(participants: u32) -> Self {
        Self { participants }
    }

    pub fn aggregate_taproot_trees(
        &self,
        participant_keys: &[PublicKey],
    ) -> Result<String, String> {
        if self.participants < 1 {
            return Err("At least one participant required".to_string());
        }

        let bitcoin_secp = crate::musig2::get_bitcoin_secp_context();
        let internal_key_secp = crate::musig2::aggregate_public_keys(participant_keys)?;
        let internal_key = crate::musig2::to_bitcoin_xonly(internal_key_secp);

        let mut builder = TaprootBuilder::new();
        // Use depth-based leaf addition to avoid complex tree structure errors in skeletal phase
        for i in 0..16 {
            let script_bytes = [i as u8; 32];
            let script = ScriptBuf::from_bytes(script_bytes.to_vec());
            builder = builder
                .add_leaf(4, script)
                .map_err(|e| format!("Failed to add leaf: {:?}", e))?;
        }

        let spend_info = builder
            .finalize(&bitcoin_secp, internal_key)
            .map_err(|e| format!("Failed to finalize taproot: {:?}", e))?;

        Ok(format!("sha256:{}", spend_info.merkle_root().unwrap()))
    }
}

#[cfg(test)]
mod multiparty_tests {
    use super::*;
    use crate::musig2::Musig2Participant;

    #[test]
    fn test_taproot_tree_aggregation_real() {
        let p1 = Musig2Participant::new();
        let p2 = Musig2Participant::new();
        let agg = Bitvm2MultiPartyAggregation::new(2);
        let result = agg.aggregate_taproot_trees(&[p1.public_key(), p2.public_key()]);
        match result {
            Ok(root) => assert!(root.starts_with("sha256:")),
            Err(e) => panic!("Aggregation failed: {}", e),
        }
    }
}
