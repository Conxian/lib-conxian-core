//! Offline capability declarations. Validation is structural, not producer authentication.
//! Registry membership, runtime health, and production qualification are separate facts.

use crate::control_model::{chain_family_for, Chain, ChainFamily, ALL_CHAINS};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CAPABILITY_SCHEMA_VERSION: u32 = 1;
pub const CHAIN_TAXONOMY_VERSION: &str = "1.0.0";
pub const MAX_MANIFEST_LIFETIME_SECONDS: u64 = 86_400;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KnownChain {
    pub chain: Chain,
    pub family: ChainFamily,
}

/// Canonical known identifiers; this list makes no runtime support claim.
pub fn known_chains() -> Vec<KnownChain> {
    ALL_CHAINS
        .iter()
        .map(|chain| KnownChain {
            chain: chain.clone(),
            family: chain_family_for(chain),
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityOperation {
    Observe,
    Verify,
    Route,
    Prepare,
    Sign,
    Broadcast,
    Recover,
    DelegatedExecute,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    NotEvidenced,
    Partial,
    Evidenced,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductionSupport {
    Unsupported,
    Conditional,
    ProductionSupported,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeState {
    Unavailable,
    Configured,
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityEvidence {
    pub api: EvidenceState,
    pub implementation: EvidenceState,
    pub integration: EvidenceState,
    pub independent_review: EvidenceState,
    pub production_support: ProductionSupport,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperationCapability {
    pub chain: Chain,
    pub network: String,
    pub operation: CapabilityOperation,
    pub runtime_state: RuntimeState,
    pub evidence: CapabilityEvidence,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityManifest {
    pub schema_version: u32,
    pub taxonomy_version: String,
    pub repository: String,
    pub revision: String,
    pub trust_root: Chain,
    pub universal_chain: bool,
    pub native_conxian_token: bool,
    /// Unix seconds. The consumer supplies its own clock to validate_at.
    pub observed_at: u64,
    pub expires_at: u64,
    pub operations: Vec<OperationCapability>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityManifestError {
    IncompatibleVersion,
    ArchitectureMismatch,
    InvalidProvenance,
    InvalidLifetime,
    TooManyOperations,
    InvalidNetwork,
    DuplicateOperation,
    InvalidEvidenceReference,
    UnqualifiedProductionClaim,
}

impl std::fmt::Display for CapabilityManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid capability manifest: {self:?}")
    }
}

impl std::error::Error for CapabilityManifestError {}

impl CapabilityManifest {
    /// Reject incompatible, stale, lossy, duplicate, and unsupported declarations.
    /// This does not authenticate the producer or verify referenced evidence.
    /// Consumers must bind the authenticated provider and deployment independently.
    pub fn validate_at(&self, now: u64) -> Result<(), CapabilityManifestError> {
        use CapabilityManifestError as E;
        if self.schema_version != CAPABILITY_SCHEMA_VERSION
            || self.taxonomy_version != CHAIN_TAXONOMY_VERSION
        {
            return Err(E::IncompatibleVersion);
        }
        if self.trust_root != Chain::Bitcoin || !self.universal_chain || self.native_conxian_token {
            return Err(E::ArchitectureMismatch);
        }
        let repository_parts: Vec<_> = self.repository.split('/').collect();
        if repository_parts.len() != 2
            || repository_parts.iter().any(|part| {
                part.is_empty()
                    || part.len() > 100
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            })
            || self.revision.len() != 40
            || !self.revision.bytes().all(|b| b.is_ascii_hexdigit())
            || self.revision.bytes().all(|b| b == b'0')
        {
            return Err(E::InvalidProvenance);
        }
        if self.observed_at == 0
            || self.observed_at > now
            || self.expires_at <= now
            || self.expires_at <= self.observed_at
            || self.expires_at - self.observed_at > MAX_MANIFEST_LIFETIME_SECONDS
        {
            return Err(E::InvalidLifetime);
        }
        if self.operations.len() > 4096 {
            return Err(E::TooManyOperations);
        }
        let mut seen = BTreeSet::new();
        for row in &self.operations {
            if row.network.is_empty()
                || row.network.len() > 64
                || !row
                    .network
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
            {
                return Err(E::InvalidNetwork);
            }
            // Serialization preserves exact chain identifiers, not coarse family aliases.
            let key = serde_json::to_string(&(&row.chain, &row.network, row.operation))
                .map_err(|_| E::InvalidProvenance)?;
            if !seen.insert(key) {
                return Err(E::DuplicateOperation);
            }
            let evidence = &row.evidence;
            if evidence.references.len() > 32
                || evidence
                    .references
                    .iter()
                    .any(|r| r.trim().is_empty() || r.len() > 2048)
            {
                return Err(E::InvalidEvidenceReference);
            }
            if evidence.production_support == ProductionSupport::ProductionSupported
                && (row.runtime_state != RuntimeState::Healthy
                    || [
                        evidence.api,
                        evidence.implementation,
                        evidence.integration,
                        evidence.independent_review,
                    ]
                    .iter()
                    .any(|s| *s != EvidenceState::Evidenced)
                    || evidence.references.is_empty())
            {
                return Err(E::UnqualifiedProductionClaim);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> CapabilityManifest {
        CapabilityManifest {
            schema_version: 1,
            taxonomy_version: CHAIN_TAXONOMY_VERSION.into(),
            repository: "Conxian/conxian-gateway".into(),
            revision: "439882ce760f4198ebfcba4b2b54c66b9b1ea991".into(),
            trust_root: Chain::Bitcoin,
            universal_chain: true,
            native_conxian_token: false,
            observed_at: 100,
            expires_at: 200,
            operations: vec![OperationCapability {
                chain: Chain::Ethereum,
                network: "sepolia".into(),
                operation: CapabilityOperation::Observe,
                runtime_state: RuntimeState::Unavailable,
                evidence: CapabilityEvidence {
                    api: EvidenceState::Evidenced,
                    implementation: EvidenceState::Partial,
                    integration: EvidenceState::NotEvidenced,
                    independent_review: EvidenceState::NotEvidenced,
                    production_support: ProductionSupport::Unsupported,
                    references: vec![],
                },
            }],
        }
    }

    #[test]
    fn taxonomy_preserves_every_canonical_chain() {
        let rows = known_chains();
        let unique: BTreeSet<_> = rows
            .iter()
            .map(|r| serde_json::to_string(&r.chain).unwrap())
            .collect();
        assert_eq!(rows.len(), ALL_CHAINS.len());
        assert_eq!(rows.len(), unique.len());
        assert!(rows.iter().any(|r| r.chain == Chain::Solana));
    }

    #[test]
    fn exact_non_bitcoin_observation_can_be_declared_without_production_support() {
        let value = manifest();
        assert_eq!(value.validate_at(150), Ok(()));
        assert_eq!(
            serde_json::from_str::<CapabilityManifest>(&serde_json::to_string(&value).unwrap())
                .unwrap(),
            value
        );
    }

    #[test]
    fn rejects_incompatible_stale_future_unbounded_and_duplicate_manifests() {
        let mut value = manifest();
        value.schema_version = 2;
        assert_eq!(
            value.validate_at(150),
            Err(CapabilityManifestError::IncompatibleVersion)
        );
        value = manifest();
        assert_eq!(
            value.validate_at(200),
            Err(CapabilityManifestError::InvalidLifetime)
        );
        assert_eq!(
            value.validate_at(99),
            Err(CapabilityManifestError::InvalidLifetime)
        );
        value.expires_at = u64::MAX;
        assert_eq!(
            value.validate_at(150),
            Err(CapabilityManifestError::InvalidLifetime)
        );
        value = manifest();
        value.operations.push(value.operations[0].clone());
        assert_eq!(
            value.validate_at(150),
            Err(CapabilityManifestError::DuplicateOperation)
        );
    }

    #[test]
    fn healthy_status_alone_cannot_establish_production_support() {
        let mut value = manifest();
        value.operations[0].runtime_state = RuntimeState::Healthy;
        value.operations[0].evidence.production_support = ProductionSupport::ProductionSupported;
        assert_eq!(
            value.validate_at(150),
            Err(CapabilityManifestError::UnqualifiedProductionClaim)
        );
    }

    #[test]
    fn rejects_lossy_family_aliases_unknown_fields_token_claims_and_invalid_networks() {
        let value = serde_json::to_string(&manifest()).unwrap();
        assert!(serde_json::from_str::<CapabilityManifest>(
            &value.replace("\"ethereum\"", "\"evm\"")
        )
        .is_err());
        assert!(serde_json::from_str::<CapabilityManifest>(&value.replacen(
            '{',
            "{\"unknown\":true,",
            1
        ))
        .is_err());
        let mut parsed = manifest();
        parsed.native_conxian_token = true;
        assert_eq!(
            parsed.validate_at(150),
            Err(CapabilityManifestError::ArchitectureMismatch)
        );
        parsed = manifest();
        parsed.operations[0].network.clear();
        assert_eq!(
            parsed.validate_at(150),
            Err(CapabilityManifestError::InvalidNetwork)
        );
    }
}
