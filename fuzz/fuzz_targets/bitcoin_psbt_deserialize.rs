#![no_main]
//! Fuzz the upstream `bitcoin::psbt::Psbt` parser with bounded input.
//!
//! This is dependency-level parser coverage. It does not exercise any removed
//! lib-conxian-core Bitcoin orchestration or Vault APIs.

use bitcoin::psbt::Psbt;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 16 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];
    let _ = Psbt::deserialize(data);
});
