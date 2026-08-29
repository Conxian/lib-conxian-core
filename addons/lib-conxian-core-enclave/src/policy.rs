//! Adapter-owned policy mappings for the exact `conxius-enclave-sdk 2.0.16`
//! release.
//!
//! These types deliberately do not extend or reinterpret Core or SDK enums.
//! The adapter owns the wire representation and the fallible policy checks at
//! the integration boundary. In particular, SDK `T4` is an observed external
//! rail tier, not a sign-capable representation of Core `ObserverOnly`.

use conxius_enclave_sdk::{
    config::Network as SdkNetwork, protocol::rails::TrustTier as SdkRailTrustTier,
};
use lib_conxian_core::control_model::TrustTier;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::AdapterError;

/// Adapter-owned wire representation of the SDK rail trust tier.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RailTrustTier {
    T1,
    T2,
    T3,
    T4,
}

impl RailTrustTier {
    /// Parses the adapter's lower-case wire representation and known SDK spellings.
    pub fn from_wire(value: &str) -> Result<Self, AdapterError> {
        match value {
            "t1" | "T1" => Ok(Self::T1),
            "t2" | "T2" => Ok(Self::T2),
            "t3" | "T3" => Ok(Self::T3),
            "t4" | "T4" => Ok(Self::T4),
            other => Err(AdapterError::UnknownRailTrustTier {
                value: other.to_owned(),
            }),
        }
    }

    /// Converts an exact SDK `2.0.16` rail tier without relying on enum layout.
    pub const fn from_sdk(value: SdkRailTrustTier) -> Self {
        match value {
            SdkRailTrustTier::T1 => Self::T1,
            SdkRailTrustTier::T2 => Self::T2,
            SdkRailTrustTier::T3 => Self::T3,
            SdkRailTrustTier::T4 => Self::T4,
        }
    }

    /// Converts to the exact SDK `2.0.16` rail tier.
    pub const fn to_sdk(self) -> SdkRailTrustTier {
        match self {
            Self::T1 => SdkRailTrustTier::T1,
            Self::T2 => SdkRailTrustTier::T2,
            Self::T3 => SdkRailTrustTier::T3,
            Self::T4 => SdkRailTrustTier::T4,
        }
    }

    /// Returns the canonical wire label used in the bound signing digest.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
            Self::T4 => "t4",
        }
    }

    const fn strength(self) -> u8 {
        match self {
            Self::T1 => 3,
            Self::T2 => 2,
            Self::T3 => 1,
            Self::T4 => 0,
        }
    }
}

impl<'de> Deserialize<'de> for RailTrustTier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_wire(&value).map_err(D::Error::custom)
    }
}

/// Adapter-owned validated pairing of a Core trust request and an observed SDK
/// rail tier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RailTrustPolicy {
    pub requested_core_tier: TrustTier,
    pub observed_sdk_tier: RailTrustTier,
}

impl RailTrustPolicy {
    /// Creates a policy and rejects an observed SDK tier weaker than the Core request.
    pub fn new(
        requested_core_tier: TrustTier,
        observed_sdk_tier: RailTrustTier,
    ) -> Result<Self, AdapterError> {
        let policy = Self {
            requested_core_tier,
            observed_sdk_tier,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Validates the policy without invoking a provider or allowing a trust downgrade.
    pub fn validate(&self) -> Result<(), AdapterError> {
        let required = match &self.requested_core_tier {
            TrustTier::Strict => RailTrustTier::T1,
            TrustTier::Managed => RailTrustTier::T2,
            TrustTier::Expedient => RailTrustTier::T3,
            // Observer-only is intentionally not mapped to SDK T4. It is an
            // observation policy and cannot authorize signing.
            TrustTier::ObserverOnly => return Ok(()),
        };

        if self.observed_sdk_tier.strength() < required.strength() {
            return Err(AdapterError::RailTrustDowngrade {
                requested: self.requested_core_tier.clone(),
                observed: self.observed_sdk_tier,
            });
        }
        Ok(())
    }

    /// Returns the SDK tier to use for a sign-capable Core request.
    pub fn signing_sdk_tier(&self) -> Result<SdkRailTrustTier, AdapterError> {
        if matches!(&self.requested_core_tier, TrustTier::ObserverOnly) {
            return Err(AdapterError::ObserverOnlyCannotSign);
        }
        self.validate()?;
        Ok(self.observed_sdk_tier.to_sdk())
    }
}

/// Maps a requested Core signing tier to the SDK rail tier it requires.
///
/// `ObserverOnly` has no SDK sign-tier mapping. Returning `T4` here would turn
/// an observation-only Core policy into an SDK policy that can still sign.
pub fn core_trust_to_sdk_rail_tier(tier: TrustTier) -> Result<SdkRailTrustTier, AdapterError> {
    match tier {
        TrustTier::Strict => Ok(SdkRailTrustTier::T1),
        TrustTier::Managed => Ok(SdkRailTrustTier::T2),
        TrustTier::Expedient => Ok(SdkRailTrustTier::T3),
        TrustTier::ObserverOnly => Err(AdapterError::ObserverOnlyCannotSign),
    }
}

/// Maps an observed SDK rail tier to an adapter-owned observation classification.
///
/// The `T4` result remains an observed external rail tier. It does not grant
/// signing permission and is never used as the reverse mapping for Core
/// `ObserverOnly`.
pub fn sdk_rail_tier_to_observation(value: SdkRailTrustTier) -> RailTrustTier {
    RailTrustTier::from_sdk(value)
}

/// Maps the observed SDK rail tier to the closest Core observation label.
///
/// This is intentionally one-way and observation-only; callers must use
/// [`RailTrustPolicy::signing_sdk_tier`] for authorization.
pub fn sdk_rail_tier_to_core_observation(value: SdkRailTrustTier) -> TrustTier {
    match value {
        SdkRailTrustTier::T1 => TrustTier::Strict,
        SdkRailTrustTier::T2 => TrustTier::Managed,
        SdkRailTrustTier::T3 => TrustTier::Expedient,
        SdkRailTrustTier::T4 => TrustTier::ObserverOnly,
    }
}

/// Validates a Core trust request against an observed SDK rail tier.
pub fn validate_rail_trust(
    requested_core_tier: TrustTier,
    observed_sdk_tier: SdkRailTrustTier,
) -> Result<RailTrustPolicy, AdapterError> {
    RailTrustPolicy::new(
        requested_core_tier,
        RailTrustTier::from_sdk(observed_sdk_tier),
    )
}

/// Adapter-owned wire representation of the exact SDK network enum.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    Mainnet,
    Testnet,
    Devnet,
}

impl NetworkPolicy {
    /// Parses the lower-case adapter wire representation and SDK spellings.
    pub fn from_wire(value: &str) -> Result<Self, AdapterError> {
        match value {
            "mainnet" | "Mainnet" => Ok(Self::Mainnet),
            "testnet" | "Testnet" => Ok(Self::Testnet),
            "devnet" | "Devnet" => Ok(Self::Devnet),
            other => Err(AdapterError::UnknownNetwork {
                value: other.to_owned(),
            }),
        }
    }

    /// Converts an exact SDK `2.0.16` network enum.
    pub const fn from_sdk(value: SdkNetwork) -> Self {
        match value {
            SdkNetwork::Mainnet => Self::Mainnet,
            SdkNetwork::Testnet => Self::Testnet,
            SdkNetwork::Devnet => Self::Devnet,
        }
    }

    /// Converts to the exact SDK `2.0.16` network enum.
    pub const fn to_sdk(self) -> SdkNetwork {
        match self {
            Self::Mainnet => SdkNetwork::Mainnet,
            Self::Testnet => SdkNetwork::Testnet,
            Self::Devnet => SdkNetwork::Devnet,
        }
    }

    /// Returns the canonical wire label used in the bound signing digest.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Devnet => "devnet",
        }
    }
}

impl<'de> Deserialize<'de> for NetworkPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_wire(&value).map_err(D::Error::custom)
    }
}

impl TryFrom<SdkNetwork> for NetworkPolicy {
    type Error = AdapterError;

    fn try_from(value: SdkNetwork) -> Result<Self, Self::Error> {
        Ok(Self::from_sdk(value))
    }
}

impl From<NetworkPolicy> for SdkNetwork {
    fn from(value: NetworkPolicy) -> Self {
        value.to_sdk()
    }
}

/// Adapter-owned policy evidence required by every signing request.
///
/// The network value is intentionally an explicit enum rather than a URL or
/// runtime endpoint. Provider selection, URLs, and network I/O remain outside
/// Core and this adapter. The rail policy carries both the Core tier requested
/// by the caller and the observed SDK rail tier that the adapter enforces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RequestPolicyContext {
    pub network: NetworkPolicy,
    pub rail: RailTrustPolicy,
}

impl RequestPolicyContext {
    /// Creates a request context after validating its rail mapping.
    pub fn new(network: NetworkPolicy, rail: RailTrustPolicy) -> Result<Self, AdapterError> {
        let context = Self { network, rail };
        context.validate()?;
        Ok(context)
    }

    /// Validates the adapter-owned enum mappings without invoking a provider.
    pub fn validate(&self) -> Result<(), AdapterError> {
        self.rail.validate()?;
        let _ = self.network.to_sdk();
        Ok(())
    }
}
