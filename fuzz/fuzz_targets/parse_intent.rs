#![no_main]
use lib_conxian_core::protocol::intent::{Fdc3Instrument, IntentManager};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];

    if let Ok(s) = std::str::from_utf8(data) {
        let instrument = Fdc3Instrument {
            ticker: s.to_string(),
            name: Some(s.to_string()),
            isin: Some(s.to_string()),
            conxian_asset_id: s.to_string(),
        };
        let _ = IntentManager::resolve_fdc3_intent(&instrument, 100, s);
    }
});
