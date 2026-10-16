//! Versioned, offline billing definitions shared by Gateway and Nexus.
//!
//! These are the existing configured schedules, not a commercial offer or a
//! proof of payment. Gateway service charges, Nexus signature quotas and ADR-004
//! settlement fees are different products and units. No tier maps automatically
//! into another product, authorizes execution or proves provider availability.

use serde::{Deserialize, Serialize};

pub const BILLING_CATALOG_SCHEMA: &str = "conxian.billing.catalog";
pub const BILLING_CATALOG_VERSION: u16 = 1;
pub const BILLING_CATALOG_JSON: &str = include_str!("../spec/billing_conformance.v1.json");

/// Existing Gateway schedule; calculations remain owned by Gateway.
pub mod gateway {
    pub const MANAGED_GATEWAY_BASE_FEE_CENTS: u64 = 20_000;
    pub const RELAY_MESSAGE_COST_CENTS: u64 = 1;
    pub const RWA_VERIFICATION_COST_CENTS: u64 = 5;
    pub const SETTLEMENT_OP_COST_CENTS: u64 = 10;
    pub const ENTERPRISE_RELAY_THRESHOLD: u64 = 100_000;
    pub const ENTERPRISE_DISCOUNT_BPS: u64 = 2_000;
    pub const BILLING_PERIOD_SECONDS: u64 = 30 * 24 * 3600;
}

/// Existing Nexus configuration. A tier value is not payment evidence.
pub mod nexus {
    use super::*;

    pub const PRO_UPGRADE_SATS: u64 = 100_000;
    pub const ENTERPRISE_UPGRADE_SATS: u64 = 1_000_000;
    pub const INVOICE_EXPIRY_SECONDS: u64 = 3_600;

    /// Configured quota/feature eligibility, independent of runtime qualification.
    /// Consumers must authenticate the tenant and verify its entitlement.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum SubscriptionTier {
        #[default]
        Free,
        Pro,
        Enterprise,
    }

    impl SubscriptionTier {
        pub fn signature_limit(&self) -> u64 {
            match self {
                Self::Free => 50_000,
                Self::Pro => 500_000,
                Self::Enterprise => 5_000_000,
            }
        }

        pub fn can_use_dlc(&self) -> bool {
            matches!(self, Self::Pro | Self::Enterprise)
        }

        pub fn can_use_zkml(&self) -> bool {
            matches!(self, Self::Enterprise)
        }

        pub fn can_use_tableland(&self) -> bool {
            matches!(self, Self::Pro | Self::Enterprise)
        }

        pub fn can_use_bitvm(&self) -> bool {
            matches!(self, Self::Enterprise)
        }

        /// ADR-004 percentage-to-floor eligibility only. This is neither a
        /// Gateway MRR discount nor proof of an active, paid subscription.
        pub fn enterprise_fee_cap(&self) -> bool {
            matches!(self, Self::Enterprise)
        }

        pub fn parse(value: &str) -> Option<Self> {
            match value {
                "free" => Some(Self::Free),
                "pro" => Some(Self::Pro),
                "enterprise" => Some(Self::Enterprise),
                _ => None,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayBillingSchedule {
    pub unit: String,
    pub period_seconds: u64,
    pub managed_base: u64,
    pub relay: u64,
    pub rwa_verification: u64,
    pub settlement_operation: u64,
    pub volume_relay_threshold: u64,
    pub volume_discount_bps: u64,
    pub volume_discount_scope: String,
    pub self_hosted_service_charge: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NexusTierSchedule {
    pub tier: nexus::SubscriptionTier,
    pub signature_limit: u64,
    pub upgrade_amount: u64,
    pub enterprise_settlement_cap_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NexusBillingSchedule {
    pub upgrade_unit: String,
    pub invoice_expiry_seconds: u64,
    pub tiers: Vec<NexusTierSchedule>,
}

/// A deterministic description of configuration. Never an authorization input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BillingCatalog {
    pub schema: String,
    pub schema_version: u16,
    pub policy_id: String,
    pub status: String,
    pub confers_entitlement: bool,
    pub cross_product_entitlement: String,
    pub settlement_policy: String,
    pub gateway: GatewayBillingSchedule,
    pub nexus: NexusBillingSchedule,
}

/// Current legacy configuration. Prices and quota behavior are unchanged.
pub fn billing_catalog() -> BillingCatalog {
    use gateway::*;
    use nexus::*;

    BillingCatalog {
        schema: BILLING_CATALOG_SCHEMA.into(),
        schema_version: BILLING_CATALOG_VERSION,
        policy_id: "legacy-billing-v1".into(),
        status: "configuration_only".into(),
        confers_entitlement: false,
        cross_product_entitlement: "unavailable".into(),
        settlement_policy: "ADR-004; separate from service charges and signature quotas".into(),
        gateway: GatewayBillingSchedule {
            unit: "usd_cents".into(),
            period_seconds: BILLING_PERIOD_SECONDS,
            managed_base: MANAGED_GATEWAY_BASE_FEE_CENTS,
            relay: RELAY_MESSAGE_COST_CENTS,
            rwa_verification: RWA_VERIFICATION_COST_CENTS,
            settlement_operation: SETTLEMENT_OP_COST_CENTS,
            volume_relay_threshold: ENTERPRISE_RELAY_THRESHOLD,
            volume_discount_bps: ENTERPRISE_DISCOUNT_BPS,
            volume_discount_scope: "base_and_usage_total".into(),
            self_hosted_service_charge: 0,
        },
        nexus: NexusBillingSchedule {
            upgrade_unit: "satoshis".into(),
            invoice_expiry_seconds: INVOICE_EXPIRY_SECONDS,
            tiers: [
                (SubscriptionTier::Free, 0),
                (SubscriptionTier::Pro, PRO_UPGRADE_SATS),
                (SubscriptionTier::Enterprise, ENTERPRISE_UPGRADE_SATS),
            ]
            .into_iter()
            .map(|(tier, upgrade_amount)| NexusTierSchedule {
                tier,
                signature_limit: tier.signature_limit(),
                upgrade_amount,
                enterprise_settlement_cap_eligible: tier.enterprise_fee_cap(),
            })
            .collect(),
        },
    }
}

/// Strict, bounded conformance check against this exact catalog revision.
/// Matching structure/configuration still does not authenticate a publisher.
pub fn validate_billing_catalog_json(json: &str) -> Result<BillingCatalog, &'static str> {
    if json.len() > 16_384 {
        return Err("billing catalog exceeds size limit");
    }
    let catalog: BillingCatalog =
        serde_json::from_str(json).map_err(|_| "invalid billing catalog")?;
    if catalog != billing_catalog() {
        return Err("billing catalog does not match this policy revision");
    }
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[cfg(feature = "sdk-blockchain")]
    #[test]
    fn opted_in_sdk_retains_exact_provider_classification_and_policy_version() {
        use crate::sdk::conxius_enclave_sdk::enclave::attestation::{
            HardwareProviderType, ATTESTATION_POLICY_DIGEST_VERSION,
        };
        assert_eq!(ATTESTATION_POLICY_DIGEST_VERSION, 2);
        for ambiguous in ["TDX", "Intel TDX", "KEYMINT", "SEV", "HSM", "YUBIKEY"] {
            assert!(HardwareProviderType::from_str_token(ambiguous).is_none());
        }
        assert_eq!(
            HardwareProviderType::from_str_token("AWS_NITRO"),
            Some(HardwareProviderType::AwsNitro)
        );
    }

    #[test]
    fn published_catalog_matches_compiled_definitions() {
        assert_eq!(
            validate_billing_catalog_json(BILLING_CATALOG_JSON).unwrap(),
            billing_catalog()
        );
        assert_eq!(
            serde_json::to_value(billing_catalog()).unwrap(),
            serde_json::from_str::<Value>(BILLING_CATALOG_JSON).unwrap()
        );
    }

    #[test]
    fn rejected_catalogs_cannot_change_prices_units_or_authority() {
        for (pointer, value) in [
            ("/schema_version", json!(2)),
            ("/policy_id", json!("unknown")),
            ("/status", json!("production_authorized")),
            ("/confers_entitlement", json!(true)),
            ("/cross_product_entitlement", json!("managed_is_enterprise")),
            ("/gateway/unit", json!("satoshis")),
            ("/gateway/managed_base", json!(2_900)),
            ("/gateway/volume_discount_scope", json!("usage_only")),
            ("/nexus/upgrade_unit", json!("X402")),
            ("/nexus/tiers/0/signature_limit", json!(5_000_000)),
            (
                "/nexus/tiers/1/enterprise_settlement_cap_eligible",
                json!(true),
            ),
        ] {
            let mut catalog = serde_json::to_value(billing_catalog()).unwrap();
            *catalog.pointer_mut(pointer).unwrap() = value;
            assert!(
                validate_billing_catalog_json(&catalog.to_string()).is_err(),
                "{pointer}"
            );
        }
    }

    #[test]
    fn malformed_duplicate_unknown_missing_and_large_inputs_are_rejected() {
        for input in [
            "",
            "{}",
            "[]",
            "null",
            "{\"schema_version\":1,\"schema_version\":1}",
        ] {
            assert!(validate_billing_catalog_json(input).is_err());
        }
        let mut catalog = serde_json::to_value(billing_catalog()).unwrap();
        catalog["gateway"]["authorization"] = json!(true);
        assert!(validate_billing_catalog_json(&catalog.to_string()).is_err());
        let duplicate = BILLING_CATALOG_JSON.replacen(
            "\"schema_version\": 1,",
            "\"schema_version\": 1, \"schema_version\": 1,",
            1,
        );
        assert!(validate_billing_catalog_json(&duplicate).is_err());
        assert!(validate_billing_catalog_json(&" ".repeat(16_385)).is_err());
    }

    #[test]
    fn existing_tier_wire_and_feature_behavior_is_preserved() {
        use nexus::SubscriptionTier as Tier;
        for (name, tier, limit, dlc, enterprise) in [
            ("free", Tier::Free, 50_000, false, false),
            ("pro", Tier::Pro, 500_000, true, false),
            ("enterprise", Tier::Enterprise, 5_000_000, true, true),
        ] {
            assert_eq!(Tier::parse(name), Some(tier));
            assert_eq!(serde_json::to_value(tier).unwrap(), json!(name));
            assert_eq!(tier.signature_limit(), limit);
            assert_eq!(tier.can_use_dlc(), dlc);
            assert_eq!(tier.can_use_tableland(), dlc);
            assert_eq!(tier.can_use_zkml(), enterprise);
            assert_eq!(tier.can_use_bitvm(), enterprise);
            assert_eq!(tier.enterprise_fee_cap(), enterprise);
        }
        assert_eq!(Tier::default(), Tier::Free);
        for alias in [
            "managed",
            "strict",
            "business",
            "Enterprise",
            " enterprise",
            "",
        ] {
            assert!(Tier::parse(alias).is_none());
            assert!(serde_json::from_value::<Tier>(json!(alias)).is_err());
        }
    }
}
