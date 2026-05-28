//! Base Layer Orchestration: rust-bitcoin and BDK
//! Aligned with CXIP 20 Section 4.0

use base64::engine::general_purpose::{
    STANDARD as BASE64_STANDARD, STANDARD_NO_PAD as BASE64_STANDARD_NO_PAD,
};
use base64::Engine;
use bdk::database::MemoryDatabase;
use bdk::{SignOptions, Wallet};
use bitcoin::consensus::deserialize;
use bitcoin::{psbt::Psbt, Network, Transaction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletDescriptorSet {
    pub external: String,
    pub change: Option<String>,
    pub network: Network,
}

impl WalletDescriptorSet {
    pub fn new(external: impl Into<String>, change: Option<String>, network: Network) -> Self {
        Self {
            external: external.into(),
            change,
            network,
        }
    }

    /// Validates descriptors by constructing an in-memory BDK wallet.
    /// This remains fully offline and performs no network calls.
    pub fn validate_offline(&self) -> anyhow::Result<()> {
        BitcoinOrchestrator::validate_descriptors(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsbtEncoding {
    Base64,
    Hex,
}

pub struct BitcoinOrchestrator;

impl BitcoinOrchestrator {
    /// Utilizes official rust-bitcoin for parsing
    pub fn parse_transaction(hex: &str) -> anyhow::Result<Transaction> {
        let decoded = hex::decode(hex).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
        let tx: Transaction =
            deserialize(&decoded).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        Ok(tx)
    }

    /// Validates wallet descriptors by constructing an in-memory BDK wallet.
    /// This is a pure/offline validation path.
    pub fn validate_descriptors(descriptor_set: &WalletDescriptorSet) -> anyhow::Result<()> {
        Self::wallet_from_descriptors(descriptor_set).map(|_| ())
    }

    /// Constructs an in-memory wallet from descriptors.
    pub fn wallet_from_descriptors(
        descriptor_set: &WalletDescriptorSet,
    ) -> anyhow::Result<Wallet<MemoryDatabase>> {
        Wallet::new(
            descriptor_set.external.as_str(),
            descriptor_set.change.as_deref(),
            descriptor_set.network,
            MemoryDatabase::new(),
        )
        .map_err(|e| anyhow::anyhow!("Descriptor validation failed: {:?}", e))
    }

    /// Imports a PSBT from either base64 or hex representation.
    pub fn import_psbt(encoded: &str) -> anyhow::Result<Psbt> {
        let encoded = encoded.trim();
        if encoded.is_empty() {
            return Err(anyhow::anyhow!("PSBT payload cannot be empty"));
        }

        if let Ok(decoded) = BASE64_STANDARD.decode(encoded) {
            if let Ok(psbt) = Psbt::deserialize(&decoded) {
                return Ok(psbt);
            }
        }

        if let Ok(decoded) = BASE64_STANDARD_NO_PAD.decode(encoded) {
            if let Ok(psbt) = Psbt::deserialize(&decoded) {
                return Ok(psbt);
            }
        }

        let decoded =
            hex::decode(encoded).map_err(|e| anyhow::anyhow!("Invalid PSBT encoding: {}", e))?;
        Psbt::deserialize(&decoded)
            .map_err(|e| anyhow::anyhow!("Invalid hex-encoded PSBT bytes: {}", e))
    }

    /// Exports a PSBT as base64 or hex.
    pub fn export_psbt(psbt: &Psbt, encoding: PsbtEncoding) -> String {
        match encoding {
            PsbtEncoding::Base64 => BASE64_STANDARD.encode(psbt.serialize()),
            PsbtEncoding::Hex => hex::encode(psbt.serialize()),
        }
    }

    /// Combines multiple PSBTs that share the same unsigned transaction.
    pub fn combine_psbts(psbts: Vec<Psbt>) -> anyhow::Result<Psbt> {
        let mut psbt_iter = psbts.into_iter();
        let mut combined = psbt_iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("At least one PSBT is required"))?;

        for psbt in psbt_iter {
            if combined.unsigned_tx != psbt.unsigned_tx {
                return Err(anyhow::anyhow!(
                    "Cannot combine PSBTs with different unsigned transactions"
                ));
            }

            combined
                .combine(psbt)
                .map_err(|e| anyhow::anyhow!("PSBT combine error: {}", e))?;
        }

        Ok(combined)
    }

    /// Signs a PSBT without attempting finalization.
    pub fn sign_psbt(wallet: &Wallet<MemoryDatabase>, psbt: &mut Psbt) -> anyhow::Result<bool> {
        let sign_opts = SignOptions {
            try_finalize: false,
            ..SignOptions::default()
        };

        wallet
            .sign(psbt, sign_opts)
            .map_err(|e| anyhow::anyhow!("Sign error: {:?}", e))
    }

    /// Finalizes a PSBT using wallet policy/signers.
    pub fn finalize_psbt(wallet: &Wallet<MemoryDatabase>, psbt: &mut Psbt) -> anyhow::Result<bool> {
        let sign_opts = SignOptions {
            try_finalize: true,
            ..SignOptions::default()
        };

        wallet
            .sign(psbt, sign_opts)
            .map_err(|e| anyhow::anyhow!("Finalize error: {:?}", e))
    }

    /// PSBT Workflow (BIP-174)
    /// Roles: Creator, Updater, Signer, Extractor
    pub fn create_psbt(
        wallet: &Wallet<MemoryDatabase>,
        recipient: bitcoin::Address,
        amount_sats: u64,
    ) -> anyhow::Result<bdk::bitcoin::psbt::PartiallySignedTransaction> {
        let mut tx_builder = wallet.build_tx();
        // Convert bitcoin v0.32 ScriptBuf to bdk (v0.30) compatible bitcoin v0.30 ScriptBuf
        let script_bytes = recipient.script_pubkey().to_bytes();
        let bdk_script = bdk::bitcoin::ScriptBuf::from(script_bytes);

        tx_builder.add_recipient(bdk_script, amount_sats);
        let (mut psbt, _details) = tx_builder
            .finish()
            .map_err(|e| anyhow::anyhow!("TxBuilder error: {:?}", e))?;

        let sign_opts = SignOptions::default();
        let _ = wallet
            .sign(&mut psbt, sign_opts)
            .map_err(|e| anyhow::anyhow!("Sign error: {:?}", e))?;

        Ok(psbt)
    }
}

/// BDK Wasm Integration (Section 4.3)
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmWallet {
        descriptor: String,
    }

    #[wasm_bindgen]
    impl WasmWallet {
        #[wasm_bindgen(constructor)]
        pub fn new(descriptor: String) -> Self {
            Self { descriptor }
        }

        #[wasm_bindgen]
        pub fn get_descriptor(&self) -> String {
            self.descriptor.clone()
        }
    }

    #[wasm_bindgen]
    pub fn init_wasm_wallet() {
        // wasm-bindgen initial bindings for BDK
    }
}

#[cfg(test)]
mod tests {
    use super::{BitcoinOrchestrator, PsbtEncoding, WalletDescriptorSet};
    use bitcoin::absolute::LockTime;
    use bitcoin::{
        consensus::serialize, psbt::Psbt, OutPoint, ScriptBuf, Transaction, TxIn, TxOut,
    };

    fn sample_unsigned_tx(output_value: u64) -> Transaction {
        Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                ..TxIn::default()
            }],
            output: vec![TxOut {
                value: output_value,
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }

    fn sample_psbt(output_value: u64) -> Psbt {
        Psbt::from_unsigned_tx(sample_unsigned_tx(output_value))
            .expect("sample unsigned tx should produce a PSBT")
    }

    #[test]
    fn descriptor_validation_success_and_failure() {
        let valid = WalletDescriptorSet::new(
            "wpkh(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)",
            None,
            bitcoin::Network::Regtest,
        );
        assert!(BitcoinOrchestrator::validate_descriptors(&valid).is_ok());

        let invalid =
            WalletDescriptorSet::new("wpkh(not-a-valid-key)", None, bitcoin::Network::Regtest);
        assert!(BitcoinOrchestrator::validate_descriptors(&invalid).is_err());
    }

    #[test]
    fn psbt_import_export_roundtrip_base64() {
        let original = sample_psbt(50_000);
        let encoded = BitcoinOrchestrator::export_psbt(&original, PsbtEncoding::Base64);

        let decoded = BitcoinOrchestrator::import_psbt(&encoded)
            .expect("base64 encoded PSBT should import successfully");

        assert_eq!(decoded, original);
    }

    #[test]
    fn psbt_import_export_roundtrip_hex() {
        let original = sample_psbt(75_000);
        let encoded = BitcoinOrchestrator::export_psbt(&original, PsbtEncoding::Hex);

        let decoded = BitcoinOrchestrator::import_psbt(&encoded)
            .expect("hex encoded PSBT should import successfully");

        assert_eq!(decoded, original);
    }

    #[test]
    fn combine_psbt_same_unsigned_tx_success() {
        let original = sample_psbt(100_000);
        let psbt_a = original.clone();
        let psbt_b = original.clone();

        let combined = BitcoinOrchestrator::combine_psbts(vec![psbt_a, psbt_b])
            .expect("PSBTs with identical unsigned tx should combine");

        assert_eq!(combined.unsigned_tx, original.unsigned_tx);
    }

    #[test]
    fn combine_psbt_different_unsigned_tx_fails() {
        let psbt_a = sample_psbt(100_000);
        let psbt_b = sample_psbt(200_000);

        let result = BitcoinOrchestrator::combine_psbts(vec![psbt_a, psbt_b]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_transaction_valid_and_invalid_paths() {
        let tx = sample_unsigned_tx(120_000);
        let tx_hex = hex::encode(serialize(&tx));

        let parsed = BitcoinOrchestrator::parse_transaction(&tx_hex)
            .expect("serialized transaction should parse successfully");
        assert_eq!(parsed, tx);

        let invalid = BitcoinOrchestrator::parse_transaction("not-hex");
        assert!(invalid.is_err());
    }
}
