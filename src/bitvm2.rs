use ark_bn254::{Bn254, Fr};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use base64::engine::general_purpose;
use base64::Engine as _;
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};

pub const ENV_BITVM2_GROTH16_VK_B64: &str = "BITVM2_GROTH16_VK_B64";

#[derive(Debug)]
pub enum Bitvm2VerifyError {
    InvalidBase64,
    InvalidHex,
    InvalidProof,
    InvalidVerifyingKey,
    InvalidPublicInput,
}

impl std::fmt::Display for Bitvm2VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    let trimmed = s.trim().strip_prefix("0x").unwrap_or(s.trim());
    let bytes = hex::decode(trimmed).map_err(|_| Bitvm2VerifyError::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Bitvm2VerifyError::InvalidHex);
    }
    Ok(bytes)
}

fn decode_hex_any(s: &str) -> Result<Vec<u8>, Bitvm2VerifyError> {
    let trimmed = s.trim().strip_prefix("0x").unwrap_or(s.trim());
    if trimmed.is_empty() {
        return Err(Bitvm2VerifyError::InvalidHex);
    }

    let normalized = if trimmed.len() % 2 == 0 {
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
    if s.starts_with("0x") {
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
    let mut inputs = vec![fr_from_be_bytes_32_canonical(&root_bytes)?];

    if let Some(values) = extra_public_inputs {
        inputs.extend(
            values
                .iter()
                .map(|v| parse_public_input(v))
                .collect::<Result<Vec<_>, _>>()?,
        );
    }

    Groth16::<Bn254>::verify_proof(pvk.as_ref(), &proof, &inputs)
        .map_err(|_| Bitvm2VerifyError::InvalidProof)
}
