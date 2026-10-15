//! Export known identifiers without claiming live provider support.
fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": lib_conxian_core::capability::CAPABILITY_SCHEMA_VERSION,
            "taxonomy_version": lib_conxian_core::capability::CHAIN_TAXONOMY_VERSION,
            "trust_root": "bitcoin",
            "universal_chain": true,
            "native_conxian_token": false,
            "claim": "known-identifiers-only",
            "chains": lib_conxian_core::capability::known_chains(),
        }))
        .expect("canonical taxonomy is serializable")
    );
}
