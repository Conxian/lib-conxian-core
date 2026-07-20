#![no_main]
//! Fuzz test for MuSig2 key aggregation using the musig2 crate.
//!
//! This fuzz target has been migrated to use the musig2 crate directly
//! instead of lib-conxian-core. For production MuSig2 with hardware
//! attestation, use conxius-enclave-sdk.
use libfuzzer_sys::fuzz_target;
use musig2::{secp, KeyAggContext};
use secp256k1::PublicKey;

const MAX_KEYS: usize = 32;
const MAX_INPUT_BYTES: usize = MAX_KEYS * 33;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];
    let mut points: Vec<secp::Point> = Vec::new();
    let chunk_iter = data.chunks_exact(33);
    for chunk in chunk_iter {
        if let Ok(pk) = PublicKey::from_slice(chunk) {
            points.push(secp::Point::from(pk));
        }
    }

    if points.len() >= 2 {
        // KeyAggContext takes owned values, not references
        if let Ok(ctx) = KeyAggContext::new(points) {
            let _aggregated_pubkey: secp::Point = ctx.aggregated_pubkey();
        }
    }
});
