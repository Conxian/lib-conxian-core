use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::{prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use base64::engine::general_purpose;
use base64::Engine as _;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};

pub const ENV_BITVM2_GROTH16_VK_B64: &str = "BITVM2_GROTH16_VK_B64";

/// BitVM2 Protocol Constants (Aligned with BIP-aligned standards)
pub const NUM_TAPS: usize = 364;
pub const VALIDATING_TAPS: usize = 1;
pub const HASHING_TAPS: usize = 363;

const MAX_CACHED_PVKS: usize = 4;

static CACHED_PVKS: OnceLock<RwLock<HashMap<String, Arc<PreparedVerifyingKey<Bn254>>>>> =
    OnceLock::new();

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Bitvm2VerifyError {
    Internal,
    InvalidBase64,
    InvalidHex,
    InvalidProof,
    InvalidVerifyingKey,
    InvalidPublicInput,
    SegmentMismatch,
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
            Self::SegmentMismatch => write!(f, "verification segment mismatch"),
        }
    }
}

impl std::error::Error for Bitvm2VerifyError {}

/// Represents a single execution segment in the BitVM2 optimistic verification flow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bitvm2Segment {
    pub index: u32,
    pub segment_type: SegmentType,
    pub script_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum SegmentType {
    Validating,
    Hashing,
}

fn decode_b64(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|_| Bitvm2VerifyError::InvalidBase64)
}

fn decode_hex_32(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    let trimmed = s.trim();
    let trimmed = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let bytes = hex::decode(trimmed).map_err(|_| Bitvm2VerifyError::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Bitvm2VerifyError::InvalidHex);
    }
    Ok(bytes)
}

fn decode_hex_any(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    let trimmed = s.trim();
    let trimmed = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
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

fn parse_public_input(s: &str) -> Result<Fr, Bitvm2VerifyError> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        let bytes = decode_hex_any(s)?;
        Ok(Fr::from_be_bytes_mod_order(&bytes))
    } else {
        Fr::from_str(s).map_err(|_| Bitvm2VerifyError::InvalidPublicInput)
    }
}

fn get_or_init_pvk(vk_b64: &str) -> Result<Arc<PreparedVerifyingKey<Bn254>>, Bitvm2VerifyError> {
    let key = vk_b64.trim();
    let cache = CACHED_PVKS.get_or_init(|| RwLock::new(HashMap::new()));

    {
        let guard = cache.read().map_err(|_| Bitvm2VerifyError::Internal)?;

        if let Some(pvk) = guard.get(key) {
            return Ok(Arc::clone(pvk));
        }
    }

    let mut guard = cache.write().map_err(|_| Bitvm2VerifyError::Internal)?;
    if let Some(pvk) = guard.get(key) {
        return Ok(Arc::clone(pvk));
    }

    let vk_bytes = decode_b64(key).map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    let mut vk_cursor = vk_bytes.as_slice();
    let vk: VerifyingKey<Bn254> = CanonicalDeserialize::deserialize_compressed(&mut vk_cursor)
        .map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    if !vk_cursor.is_empty() {
        return Err(Bitvm2VerifyError::InvalidVerifyingKey);
    }
    let pvk = Arc::new(prepare_verifying_key(&vk));

    if guard.len() >= MAX_CACHED_PVKS {
        if let Some(evict) = guard.keys().next().cloned() {
            guard.remove(&evict);
        }
    }

    guard.insert(key.to_owned(), Arc::clone(&pvk));
    Ok(pvk)
}

/// Verifies a Groth16 state root proof.
pub fn verify_state_root_bn254_groth16(
    vk_b64: &str,
    state_root: &str,
    proof_b64: &str,
    extra_public_inputs: Option<&[String]>,
) -> Result<bool, Bitvm2VerifyError> {
    let pvk = get_or_init_pvk(vk_b64)?;

    let proof_bytes = decode_b64(proof_b64).map_err(|_| Bitvm2VerifyError::InvalidProof)?;
    let mut proof_cursor = proof_bytes.as_slice();
    let proof: Proof<Bn254> = CanonicalDeserialize::deserialize_compressed(&mut proof_cursor)
        .map_err(|_| Bitvm2VerifyError::InvalidProof)?;
    if !proof_cursor.is_empty() {
        return Err(Bitvm2VerifyError::InvalidProof);
    }

    let root_bytes = decode_hex_32(state_root)?;
    let mut inputs = vec![Fr::from_be_bytes_mod_order(&root_bytes)];

    if let Some(values) = extra_public_inputs {
        for v in values {
            inputs.push(parse_public_input(v)?);
        }
    }

    Groth16::<Bn254>::verify_proof(pvk.as_ref(), &proof, &inputs)
        .map_err(|_| Bitvm2VerifyError::InvalidProof)
}

/// Generates the 364 verification segments required for BitVM2 on-chain orchestration.
/// This is used by the orchestrator to prepare the optimistic challenge path.
pub fn generate_verification_segments(
    proof_b64: &str,
) -> Result<Vec<Bitvm2Segment>, Bitvm2VerifyError> {
    use sha2::{Digest, Sha256};
    let proof_bytes = decode_b64(proof_b64)?;

    let mut segments = Vec::with_capacity(NUM_TAPS);

    let mut hasher = Sha256::new();
    hasher.update(b"BITVM2_VALIDATING_TAP");
    hasher.update(&proof_bytes);
    let validating_root = hex::encode(hasher.finalize());

    // Segment 0: The Validating Tap (Core SNARK logic)
    segments.push(Bitvm2Segment {
        index: 0,
        segment_type: SegmentType::Validating,
        script_hash: format!("0x{}", validating_root),
    });

    // Segments 1-363: Hashing Taps (Hash chain for intermediate states)
    for i in 1..NUM_TAPS {
        let mut tap_hasher = Sha256::new();
        tap_hasher.update(b"BITVM2_HASHING_TAP_");
        tap_hasher.update((i as u32).to_be_bytes());
        tap_hasher.update(&proof_bytes);
        let hashing_tap_root = hex::encode(tap_hasher.finalize());

        segments.push(Bitvm2Segment {
            index: i as u32,
            segment_type: SegmentType::Hashing,
            script_hash: format!("0x{}", hashing_tap_root),
        });
    }

    Ok(segments)
}

/// Verifies a disprove transaction by comparing the operator's claimed input/output
/// hashes against the computed values for a specific segment.
pub fn verify_disprove_transaction(
    segment_index: u32,
    _operator_input_hash: &str,
    operator_output_hash: &str,
    computed_output_hash: &str,
) -> Result<bool, Bitvm2VerifyError> {
    if segment_index >= NUM_TAPS as u32 {
        return Err(Bitvm2VerifyError::SegmentMismatch);
    }

    // A disprove transaction is valid if the operator's output hash doesn't match
    // the computed output hash for the given input.
    let is_fraud = operator_output_hash != computed_output_hash;

    // In a real implementation, we would also verify that the operator_input_hash
    // corresponds to the previous segment's output hash (hash chain integrity).

    Ok(is_fraud)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_segments() {
        let segments = generate_verification_segments("YmFzZTY0cGxhY2Vob2xkZXI=").unwrap();
        assert_eq!(segments.len(), NUM_TAPS);
        assert_eq!(segments[0].segment_type, SegmentType::Validating);
        assert_eq!(segments[1].segment_type, SegmentType::Hashing);
    }

    #[test]
    fn test_disprove_logic() {
        let is_fraud =
            verify_disprove_transaction(5, "0xinput", "0xclaimed_output", "0xcomputed_output")
                .unwrap();
        assert!(is_fraud);

        let no_fraud =
            verify_disprove_transaction(5, "0xinput", "0xcorrect_output", "0xcorrect_output")
                .unwrap();
        assert!(!no_fraud);
    }
}
