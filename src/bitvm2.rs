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

static CACHED_PVKS: OnceLock<RwLock<HashMap<String, Arc<PreparedVerifyingKey<Bn254>>>>> =
    OnceLock::new();

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

fn parse_public_input(s: &str) -> Result<Fr, Bitvm2VerifyError> {
    let s = s.trim();
    if s.starts_with("0x") {
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
        let guard = cache
            .read()
            .map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;

        if let Some(pvk) = guard.get(key) {
            return Ok(Arc::clone(pvk));
        }
    }

    let vk_bytes = decode_b64(key).map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    let mut vk_cursor = vk_bytes.as_slice();
    let vk: VerifyingKey<Bn254> = CanonicalDeserialize::deserialize_compressed(&mut vk_cursor)
        .map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    if !vk_cursor.is_empty() {
        return Err(Bitvm2VerifyError::InvalidVerifyingKey);
    }
    let pvk = Arc::new(prepare_verifying_key(&vk));

    let mut guard = cache
        .write()
        .map_err(|_| Bitvm2VerifyError::InvalidVerifyingKey)?;
    let entry = guard
        .entry(key.to_owned())
        .or_insert_with(|| Arc::clone(&pvk));
    Ok(Arc::clone(entry))
}

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
