#![no_main]
use libfuzzer_sys::fuzz_target;
use lib_conxian_core::musig2;
use secp256k1::PublicKey;

fuzz_target!(|data: &[u8]| {
    let mut pubkeys = Vec::new();
    let mut chunk_iter = data.chunks_exact(33);
    while let Some(chunk) = chunk_iter.next() {
        if let Ok(pk) = PublicKey::from_slice(chunk) {
            pubkeys.push(pk);
        }
    }

    if !pubkeys.is_empty() {
        let _ = musig2::aggregate_public_keys(&pubkeys);
    }
});
