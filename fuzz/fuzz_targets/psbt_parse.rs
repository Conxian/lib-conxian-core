#![no_main]

use libfuzzer_sys::fuzz_target;
use lib_conxian_core::bitcoin::BitcoinOrchestrator;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = BitcoinOrchestrator::parse_transaction(s);
        let _ = BitcoinOrchestrator::import_psbt(s);
    }
});
