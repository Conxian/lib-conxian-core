#![no_main]
use lib_conxian_core::musig2;
use libfuzzer_sys::fuzz_target;
use secp256k1::PublicKey;

fuzz_target!(|data: &[u8]| {
    let mut pubkeys = Vec::new();
    let chunk_iter = data.chunks_exact(33);
    for chunk in chunk_iter {
        if let Ok(pk) = PublicKey::from_slice(chunk) {
            pubkeys.push(pk);
        }
    }

    if !pubkeys.is_empty() {
        let _ = musig2::aggregate_public_keys(&pubkeys);
    }
});
