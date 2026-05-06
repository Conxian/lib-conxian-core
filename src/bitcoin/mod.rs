//! Base Layer Orchestration: rust-bitcoin and BDK
//! Aligned with CXIP 20 Section 4.0

use bdk::database::MemoryDatabase;
use bdk::{SignOptions, Wallet};
use bitcoin::consensus::deserialize;
use bitcoin::Transaction;

pub struct BitcoinOrchestrator;

impl BitcoinOrchestrator {
    /// Utilizes official rust-bitcoin for parsing
    pub fn parse_transaction(hex: &str) -> anyhow::Result<Transaction> {
        let decoded = hex::decode(hex).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
        let tx: Transaction =
            deserialize(&decoded).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        Ok(tx)
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
