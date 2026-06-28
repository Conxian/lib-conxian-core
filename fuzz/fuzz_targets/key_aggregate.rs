#![no_main]

use libfuzzer_sys::fuzz_target;
use lib_conxian_core::musig2::aggregate_public_keys;
use secp256k1::PublicKey;

fuzz_target!(|data: &[u8]| {
    // Interpret random bytes as 33-byte compressed public keys
    let keys: Vec<PublicKey> = data
        .chunks(33)
        .filter_map(|chunk| PublicKey::from_slice(chunk).ok())
        .collect();

    if keys.len() >= 2 {
        let _ = aggregate_public_keys(&keys);
    }

    // Also try as 32-byte x-only keys (shorter slices)
    if data.len() >= 64 {
        let key_bytes = [&data[..32], &data[32..]];
        let keys: Vec<PublicKey> = key_bytes
            .iter()
            .filter_map(|chunk| PublicKey::from_slice(chunk).ok())
            .collect();
        if keys.len() >= 2 {
            let _ = aggregate_public_keys(&keys);
        }
    }
});
