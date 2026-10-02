//! Canonical settlement fee model (ADR-004).
//!
//! Single source of truth for the dynamic hybrid fee floor: a rail-specific
//! flat satoshi floor composed with a logarithmic 30-day volume decay and a
//! clamped system-load factor. Mirrors the `@conxian/market-sdk`
//! `calculateDynamicFee` reference so the Rust and TypeScript surfaces stay
//! aligned.

use serde::{Deserialize, Serialize};

use crate::control_model::TrustTier;

/// Settlement rails — the destination chains a settlement can settle on.
///
/// Distinct from the enclave-sdk *bridge* rails (Bisq/Boltz/Changelly/NTT/
/// Wormhole), which are cross-chain transport protocols. See ADR-004 §3.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementRail {
    Statechain,
    Sbtc,
    Rgb,
    Babylon,
    Fedimint,
    Lightning,
    AlexStacks,
    EvmErc8183,
}

/// Logarithmic 30-day volume decay tiers (ADR-004 §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VolumeDecayTier {
    /// 2.00% — launch / low volume.
    #[default]
    Tier1,
    /// 1.50%.
    Tier2,
    /// 0.75%.
    Tier3,
    /// 0.25% — high-velocity M2M.
    Tier4,
}

impl VolumeDecayTier {
    /// Raw basis-point rate for this tier (1 bps = 0.01%).
    pub const fn raw_basis_points(self) -> u32 {
        match self {
            Self::Tier1 => 200,
            Self::Tier2 => 150,
            Self::Tier3 => 75,
            Self::Tier4 => 25,
        }
    }
}

/// Minimum percentage floor (bps) the decayed rate can never fall below.
pub const MIN_PERCENTAGE_FLOOR_BPS: u32 = 10;

/// System-load factor bounds (ADR-004 §3.3).
pub const SYSTEM_LOAD_FACTOR_MIN: f64 = 1.0;
pub const SYSTEM_LOAD_FACTOR_MAX: f64 = 3.0;

/// Rail-specific flat satoshi floors (ADR-004 §3.1).
///
/// Lightning `base_fee` analog: prices the fixed overhead of settling on a
/// rail (interchange-plus = cost + margin), independent of the settled amount.
/// Current values are placeholders pending measured per-rail cost (G8 rail
/// calibration); see `docs/FEE_MODEL_BENCHMARK.md`.
pub fn rail_default_flat_floor(rail: SettlementRail) -> u64 {
    match rail {
        SettlementRail::Lightning => 10,
        SettlementRail::Statechain => 25,
        SettlementRail::Fedimint => 25,
        SettlementRail::Rgb => 20,
        SettlementRail::Sbtc => 50,
        SettlementRail::AlexStacks => 50,
        SettlementRail::Babylon => 50,
        SettlementRail::EvmErc8183 => 100,
    }
}

/// Decayed basis-point rate for a volume tier, never below the percentage floor.
pub fn volume_decayed_bps(tier: VolumeDecayTier) -> u32 {
    tier.raw_basis_points().max(MIN_PERCENTAGE_FLOOR_BPS)
}

/// Fee distribution targets (50/30/20) — ADR-004 §3.4.
pub const FEE_DISTRIBUTION_OPS_BPS: u64 = 50;
pub const FEE_DISTRIBUTION_FOUNDERS_BPS: u64 = 30;

/// 50/30/20 fee distribution split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeDistribution {
    pub operations_sat: u64,
    pub founders_sat: u64,
    pub ecosystem_sat: u64,
}

/// Inputs to [`calculate_dynamic_fee`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeOptions {
    pub trust_tier: TrustTier,
    pub rail: SettlementRail,
    pub amount_sat: u64,
    #[serde(default)]
    pub volume_decay_tier: VolumeDecayTier,
    #[serde(default = "default_system_load_factor")]
    pub system_load_factor: f64,
    /// When true, the percentage component is replaced by the flat floor — the
    /// subscription/committed-use pricing link to nexus
    /// `SubscriptionTier::Enterprise` (G5). See `docs/FEE_MODEL_BENCHMARK.md`.
    #[serde(default)]
    pub enterprise_subscription_cap: bool,
}

/// Default system-load factor (no congestion).
pub const fn default_system_load_factor() -> f64 {
    1.0
}

/// Result of a dynamic fee calculation (ADR-004 §3.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicFeeResult {
    pub trust_tier: TrustTier,
    pub rail: SettlementRail,
    pub amount_sat: u64,
    pub percentage_fee_sat: u64,
    pub flat_floor_sat: u64,
    pub effective_fee_sat: u64,
    pub effective_bps: u32,
    pub system_load_factor: f64,
    pub volume_decay_tier: VolumeDecayTier,
    pub distribution: FeeDistribution,
}

/// Error returned when a fee calculation is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeError {
    /// Settlement is disabled for the observer-only trust tier.
    ObserverOnlyTier,
}

impl std::fmt::Display for FeeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObserverOnlyTier => write!(f, "settlement disabled for ObserverOnly tier"),
        }
    }
}

impl std::error::Error for FeeError {}

/// Calculate a dynamic settlement fee (ADR-004).
///
/// `effective_fee_sat = max(percentage_fee_sat, flat_floor_sat) * load_factor`,
/// where `percentage_fee_sat = amount_sat * decayed_bps / 10_000`. The
/// enterprise subscription cap replaces the percentage component with the flat
/// floor. The load factor is clamped to `[1.0, 3.0]` and quantised to two
/// decimal places before scaling, matching the TypeScript reference.
pub fn calculate_dynamic_fee(options: FeeOptions) -> Result<DynamicFeeResult, FeeError> {
    if options.trust_tier == TrustTier::ObserverOnly {
        return Err(FeeError::ObserverOnlyTier);
    }

    let load_factor = options
        .system_load_factor
        .clamp(SYSTEM_LOAD_FACTOR_MIN, SYSTEM_LOAD_FACTOR_MAX);
    let decay_bps = volume_decayed_bps(options.volume_decay_tier);
    let flat_floor = rail_default_flat_floor(options.rail);

    let percentage_fee_sat = (options.amount_sat as u128 * decay_bps as u128 / 10_000) as u64;

    let base_fee_sat = if options.enterprise_subscription_cap {
        flat_floor
    } else {
        percentage_fee_sat.max(flat_floor)
    };

    // Quantise the load factor to 2 decimal places before scaling (mirrors the
    // TypeScript `BigInt(Math.round(loadFactor * 100)) / 100n`).
    let load_scale = (load_factor * 100.0).round() as u128;
    let effective_fee_sat = (base_fee_sat as u128 * load_scale / 100) as u64;

    let operations_sat = effective_fee_sat * FEE_DISTRIBUTION_OPS_BPS / 100;
    let founders_sat = effective_fee_sat * FEE_DISTRIBUTION_FOUNDERS_BPS / 100;
    let ecosystem_sat = effective_fee_sat - operations_sat - founders_sat;

    let effective_bps = if options.amount_sat == 0 {
        decay_bps
    } else {
        ((effective_fee_sat as u128 * 10_000) / options.amount_sat as u128) as u32
    };

    Ok(DynamicFeeResult {
        trust_tier: options.trust_tier,
        rail: options.rail,
        amount_sat: options.amount_sat,
        percentage_fee_sat,
        flat_floor_sat: flat_floor,
        effective_fee_sat,
        effective_bps,
        system_load_factor: load_factor,
        volume_decay_tier: options.volume_decay_tier,
        distribution: FeeDistribution {
            operations_sat,
            founders_sat,
            ecosystem_sat,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(rail: SettlementRail, amount_sat: u64) -> FeeOptions {
        FeeOptions {
            trust_tier: TrustTier::Strict,
            rail,
            amount_sat,
            volume_decay_tier: VolumeDecayTier::Tier1,
            system_load_factor: 1.0,
            enterprise_subscription_cap: false,
        }
    }

    #[test]
    fn rail_floors_are_ordered() {
        assert_eq!(rail_default_flat_floor(SettlementRail::Lightning), 10);
        assert_eq!(rail_default_flat_floor(SettlementRail::Rgb), 20);
        assert_eq!(rail_default_flat_floor(SettlementRail::Statechain), 25);
        assert_eq!(rail_default_flat_floor(SettlementRail::Sbtc), 50);
        assert_eq!(rail_default_flat_floor(SettlementRail::EvmErc8183), 100);
    }

    #[test]
    fn volume_decay_tiers() {
        assert_eq!(volume_decayed_bps(VolumeDecayTier::Tier1), 200);
        assert_eq!(volume_decayed_bps(VolumeDecayTier::Tier2), 150);
        assert_eq!(volume_decayed_bps(VolumeDecayTier::Tier3), 75);
        assert_eq!(volume_decayed_bps(VolumeDecayTier::Tier4), 25);
    }

    #[test]
    fn micro_payment_hits_flat_floor() {
        // 5 sat Lightning settlement: percentage is 0 sat, floor is 10 sat.
        let r = calculate_dynamic_fee(opts(SettlementRail::Lightning, 5)).unwrap();
        assert_eq!(r.percentage_fee_sat, 0);
        assert_eq!(r.effective_fee_sat, 10);
        assert_eq!(r.effective_bps, 20_000);
    }

    #[test]
    fn percentage_dominates_large_settlement() {
        // 1_000_000 sat Tier1 (200 bps) = 20_000 sat percentage.
        let r = calculate_dynamic_fee(opts(SettlementRail::Lightning, 1_000_000)).unwrap();
        assert_eq!(r.percentage_fee_sat, 20_000);
        assert_eq!(r.effective_fee_sat, 20_000);
        assert_eq!(r.effective_bps, 200);
    }

    #[test]
    fn enterprise_cap_uses_flat_floor() {
        let mut o = opts(SettlementRail::Sbtc, 1_000_000);
        o.enterprise_subscription_cap = true;
        let r = calculate_dynamic_fee(o).unwrap();
        assert_eq!(r.effective_fee_sat, 50);
    }

    #[test]
    fn load_factor_clamped_and_quantised() {
        let mut o = opts(SettlementRail::Lightning, 5);
        o.system_load_factor = 2.5;
        // base 10 * round(2.5*100)/100 = 25.
        assert_eq!(calculate_dynamic_fee(o).unwrap().effective_fee_sat, 25);

        let mut o = opts(SettlementRail::Lightning, 5);
        o.system_load_factor = 5.0; // clamped to 3.0
        assert_eq!(calculate_dynamic_fee(o).unwrap().effective_fee_sat, 30);
    }

    #[test]
    fn observer_only_rejected() {
        let mut o = opts(SettlementRail::Lightning, 100);
        o.trust_tier = TrustTier::ObserverOnly;
        assert_eq!(
            calculate_dynamic_fee(o),
            Err(FeeError::ObserverOnlyTier)
        );
    }

    #[test]
    fn distribution_conserves_total() {
        let r = calculate_dynamic_fee(opts(SettlementRail::Babylon, 123_456)).unwrap();
        let d = r.distribution;
        assert_eq!(d.operations_sat + d.founders_sat + d.ecosystem_sat, r.effective_fee_sat);
        // 50/30/20 of 6173 sat (200 bps on 123_456 = 2469 sat, floor 50 → 2469).
        assert_eq!(d.operations_sat, r.effective_fee_sat * 50 / 100);
        assert_eq!(d.founders_sat, r.effective_fee_sat * 30 / 100);
    }
}
