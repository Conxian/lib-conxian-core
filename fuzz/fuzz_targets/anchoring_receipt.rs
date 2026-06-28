#![no_main]

use libfuzzer_sys::fuzz_target;
use lib_conxian_core::anchoring::AnchoringReceipt;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<AnchoringReceipt>(data);
});
