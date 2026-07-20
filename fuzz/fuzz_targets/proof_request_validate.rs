#![no_main]

use lib_conxian_core::ProofVerificationRequest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(request) = serde_json::from_slice::<ProofVerificationRequest>(data) {
        let _ = request.validate();
    }
});
