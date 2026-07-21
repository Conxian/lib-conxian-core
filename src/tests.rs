#[cfg(test)]
mod cxip20_architecture_tests {
    use crate::crypto::{
        AdaptorSignature, CryptoStubError, WitnessEncryption, WitnessEncryptionError, PVDE,
    };
    use crate::enclave::ZKCompliance;
    use crate::lightning::LightningNode;
    use crate::rgb::{RGBError, RGBExecutionMode, RGBRuntime, RGBSkeletonAdapter};
    use crate::stacks::{SBTCBridge, StacksNakamoto};

    #[test]
    fn test_enclave_zkc_logic() {
        assert!(ZKCompliance::verify_aml_stateless("id_comm", "tx_meta"));
    }

    #[test]
    fn test_advanced_crypto_stubs() {
        let puzzle =
            PVDE::generate_puzzle(1000, b"secret").expect("PVDE should produce deterministic hash");
        assert_eq!(puzzle.len(), 64);
        assert!(!puzzle.contains("secret"));

        let witness_err = WitnessEncryption::encrypt_to_bitcoin_finality(6, b"data")
            .expect_err("Witness encryption stub should fail closed");
        assert_eq!(
            witness_err,
            CryptoStubError::NotImplemented("WitnessEncryption::encrypt_to_bitcoin_finality")
        );
        let witness_msg = witness_err.to_string();
        assert!(!witness_msg.contains("data"));
        assert!(!witness_msg.contains("64617461"));

        let adaptor_err = AdaptorSignature::create_adaptor_signature("sec", "msg")
            .expect_err("Adaptor signature should reject invalid key material");
        assert_eq!(adaptor_err, CryptoStubError::InvalidKey);
        let adaptor_msg = adaptor_err.to_string();
        assert!(!adaptor_msg.contains("sec"));
        assert!(!adaptor_msg.contains("msg"));
    }

    #[test]
    fn test_witness_encryption_placeholder_does_not_leak_plaintext() {
        let payload = b"highly-sensitive-data";
        let err = WitnessEncryption::encrypt_to_bitcoin_finality(6, payload)
            .expect_err("Witness encryption should be fail-closed placeholder");

        let msg = err.to_string();
        assert!(!msg.contains("highly-sensitive-data"));
        assert!(!msg.contains(&hex::encode(payload)));
    }

    #[test]
    fn test_witness_encryption_try_api_reports_unimplemented() {
        let result = WitnessEncryption::try_encrypt_to_bitcoin_finality(6, b"data");
        assert_eq!(result, Err(WitnessEncryptionError::Unimplemented));
    }

    #[test]
    fn test_lightning_advanced_features() {
        let offer_result = LightningNode::create_bolt12_offer(50000, "invoice");
        assert!(offer_result.is_err()); // Currently fails closed
        assert!(LightningNode::request_jit_channel(
            "0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166"
        )
        .is_ok());
        let channel_id = [1u8; 32];
        assert!(LightningNode::initiate_splicing(&channel_id, 1000).is_ok());
    }

    #[test]
    fn test_stacks_nakamoto_sbtc() {
        assert!(StacksNakamoto::verify_bitcoin_finality(1));
        let bridge = SBTCBridge::new();
        use crate::stacks::StacksAdapter;
        assert!(bridge.initiate_peg_in(100000, "btc_txid").is_ok());
        assert!(bridge.initiate_peg_out(100000, "ST...").is_ok());
    }

    #[test]
    fn test_rgb_csv_logic() {
        let runtime = RGBRuntime::new(RGBExecutionMode::Active, RGBSkeletonAdapter);
        assert!(matches!(
            runtime.validate_transition("state_transition"),
            Err(RGBError::Unsupported { .. })
        ));
        assert!(matches!(
            runtime.verify_seal("utxo:0", "comm"),
            Err(RGBError::Unsupported { .. })
        ));
    }
}

#[cfg(test)]
mod additional_protocol_tests {
    use crate::bitcoin::SilentPaymentScanner;
    use crate::fedimint::FedimintAdapter;
    use crate::protocol::dlc::{DlcError, DlcManager};
    use secp256k1::Secp256k1;

    #[test]
    fn test_fedimint_unblinding_verification() {
        let secret = b"ecash-secret-32-byte-long-input-";
        let bf = &[0x01; 32];
        let blinded = FedimintAdapter::blind_note(secret, bf);
        assert!(FedimintAdapter::verify_unblinded(&blinded, bf, secret));
        assert!(!FedimintAdapter::verify_unblinded(
            &blinded,
            &[0x02; 32],
            secret
        ));
    }

    #[test]
    fn test_silent_payment_scanning_uniqueness() {
        let secp = Secp256k1::new();
        let (_sk1, pk1) = secp.generate_keypair(&mut secp256k1::rand::rng());
        let (_sk2, pk2) = secp.generate_keypair(&mut secp256k1::rand::rng());

        let scan_key = [0x01; 32];
        let spend_pk = [0x02; 33];

        let found1 = SilentPaymentScanner::scan_transaction(&[pk1], &scan_key, &spend_pk);
        let found2 = SilentPaymentScanner::scan_transaction(&[pk1], &scan_key, &spend_pk);
        assert_eq!(found1, found2);

        let found3 = SilentPaymentScanner::scan_transaction(&[pk2], &scan_key, &spend_pk);
        assert_ne!(found1, found3);
    }

    #[test]
    fn test_dlc_verification_logic() {
        let oracle_pk = vec![0x02; 33];
        let outcome = [0xbb; 32];
        let intent = DlcManager::create_intent(&oracle_pk, 50000, outcome, 2000);

        assert_eq!(
            DlcManager::verify_execution(&intent, &[0x01; 32]),
            Err(DlcError::UnsupportedExecutionVerification)
        );
        assert_eq!(
            DlcManager::verify_execution(&intent, &[]),
            Err(DlcError::UnsupportedExecutionVerification)
        );
    }
}
