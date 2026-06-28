#![no_main]

use libfuzzer_sys::fuzz_target;
use lib_conxian_core::bitvm2::verify_state_root_bn254_groth16;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // All same arg
        let _ = verify_state_root_bn254_groth16(s, s, s, None);
    }

    // Split into three strings for different arguments
    let len = data.len();
    if len >= 6 {
        let third = len / 3;
        let s1 = std::str::from_utf8(&data[..third]).ok();
        let s2 = std::str::from_utf8(&data[third..2 * third]).ok();
        let s3 = std::str::from_utf8(&data[2 * third..]).ok();
        if let (Some(vk), Some(root), Some(proof)) = (s1, s2, s3) {
            let _ = verify_state_root_bn254_groth16(vk, root, proof, None);
        }
    }
});
