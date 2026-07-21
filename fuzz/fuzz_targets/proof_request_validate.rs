#![no_main]

use lib_conxian_core::ProofVerificationRequest;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 16 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];

    if let Ok(request) = serde_json::from_slice::<ProofVerificationRequest>(data) {
        let _ = request.validate();
    }
});
