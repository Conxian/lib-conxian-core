#![no_main]
//! Fuzz bounded construction and verification of the core BIP-322 message type.
//!
//! Invalid addresses, malformed base64, and a `false` verification result are
//! expected outcomes. This target is for panic resistance, not signature
//! validity testing.

use lib_conxian_core::bitcoin::bip322::{Bip322Bridge, Bip322Message};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_FIELD_BYTES: usize = 1024;

fn bounded_string(bytes: &[u8]) -> String {
    let bytes = &bytes[..bytes.len().min(MAX_FIELD_BYTES)];
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn split_fields(data: &[u8]) -> (&[u8], &[u8], &[u8]) {
    if data.contains(&b'\n') {
        let mut fields = data.splitn(3, |byte| *byte == b'\n');
        (
            fields.next().unwrap_or_default(),
            fields.next().unwrap_or_default(),
            fields.next().unwrap_or_default(),
        )
    } else {
        let third = data.len() / 3;
        let two_thirds = third * 2;
        (
            &data[..third],
            &data[third..two_thirds],
            &data[two_thirds..],
        )
    }
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_BYTES)];
    let (message, address, signature) = split_fields(data);
    let message = Bip322Message {
        message: bounded_string(message),
        address: bounded_string(address),
        signature: bounded_string(signature),
    };

    let _ = Bip322Bridge::verify_message(&message);
});
