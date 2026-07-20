#![no_main]
//! Fuzz bounded JSON deserialization for the core anchoring receipt model.

use lib_conxian_core::anchoring::AnchoringReceipt;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 16 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];
    let _ = serde_json::from_slice::<AnchoringReceipt>(data);
});
