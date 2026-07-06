#![no_main]
use lib_conxian_core::protocol::intent::{Fdc3Instrument, IntentManager};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
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
