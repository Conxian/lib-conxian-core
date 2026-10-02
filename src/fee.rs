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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
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

    /// Map a rolling 30-day settlement volume (sats) to a decay tier.
    pub fn from_monthly_volume(volume_sat: u64) -> Self {
        if volume_sat < TIER2_MIN_MONTHLY_SATS {
            Self::Tier1
        } else if volume_sat < TIER3_MIN_MONTHLY_SATS {
            Self::Tier2
        } else if volume_sat < TIER4_MIN_MONTHLY_SATS {
            Self::Tier3
        } else {
            Self::Tier4
        }
    }

    /// Suggest a tier with hysteresis (ADR-004 §4.2).
    ///
    /// Moves at most one tier per call and only once the volume has cleared a
    /// boundary by [`TIER_HYSTERESIS_BPS`], so a client oscillating near a
    /// threshold does not thrash between rates. Repeated calls converge to the
    /// raw tier.
    pub fn suggest_with_hysteresis(volume_sat: u64, current: Self) -> Self {
        let raw = Self::from_monthly_volume(volume_sat);
        if raw == current {
            return current;
        }
        if raw > current {
            let next = current.next();
            let band = next.lower_bound() + next.lower_bound() * TIER_HYSTERESIS_BPS / 10_000;
            if volume_sat >= band {
                next
            } else {
                current
            }
        } else {
            let band = current.lower_bound() - current.lower_bound() * TIER_HYSTERESIS_BPS / 10_000;
            if volume_sat <= band {
                current.prev()
            } else {
                current
            }
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Tier1 => Self::Tier2,
            Self::Tier2 => Self::Tier3,
            Self::Tier3 | Self::Tier4 => Self::Tier4,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Tier1 | Self::Tier2 => Self::Tier1,
            Self::Tier3 => Self::Tier2,
            Self::Tier4 => Self::Tier3,
        }
    }

    fn lower_bound(self) -> u64 {
        match self {
            Self::Tier1 => 0,
            Self::Tier2 => TIER2_MIN_MONTHLY_SATS,
            Self::Tier3 => TIER3_MIN_MONTHLY_SATS,
            Self::Tier4 => TIER4_MIN_MONTHLY_SATS,
        }
    }
}

/// Minimum percentage floor (bps) the decayed rate can never fall below.
pub const MIN_PERCENTAGE_FLOOR_BPS: u32 = 10;

/// System-load factor bounds (ADR-004 §3.3).
pub const SYSTEM_LOAD_FACTOR_MIN: f64 = 1.0;
pub const SYSTEM_LOAD_FACTOR_MAX: f64 = 3.0;

/// Monthly settlement-volume thresholds (sats) between decay tiers (ADR-004 §4.2).
pub const TIER2_MIN_MONTHLY_SATS: u64 = 10_000_000; // ~0.1 BTC
pub const TIER3_MIN_MONTHLY_SATS: u64 = 100_000_000; // ~1 BTC
pub const TIER4_MIN_MONTHLY_SATS: u64 = 1_000_000_000; // ~10 BTC

/// Hysteresis band (bps) around tier boundaries to prevent rate thrashing.
pub const TIER_HYSTERESIS_BPS: u64 = 500; // 5%

/// Rail-specific flat satoshi floors (ADR-004 §3.1).
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
    fn volume_to_tier_mapping() {
        assert_eq!(VolumeDecayTier::from_monthly_volume(0), VolumeDecayTier::Tier1);
        assert_eq!(VolumeDecayTier::from_monthly_volume(9_999_999), VolumeDecayTier::Tier1);
        assert_eq!(VolumeDecayTier::from_monthly_volume(10_000_000), VolumeDecayTier::Tier2);
        assert_eq!(VolumeDecayTier::from_monthly_volume(100_000_000), VolumeDecayTier::Tier3);
        assert_eq!(VolumeDecayTier::from_monthly_volume(1_000_000_000), VolumeDecayTier::Tier4);
    }

    #[test]
    fn hysteresis_prevents_thrashing() {
        // Above the Tier2 boundary (10M) but below the +5% band (10.5M): stay.
        assert_eq!(
            VolumeDecayTier::suggest_with_hysteresis(10_200_000, VolumeDecayTier::Tier1),
            VolumeDecayTier::Tier1
        );
        // Above the band: upgrade.
        assert_eq!(
            VolumeDecayTier::suggest_with_hysteresis(10_600_000, VolumeDecayTier::Tier1),
            VolumeDecayTier::Tier2
        );
        // Below the Tier2 boundary (10M) but above the -5% band (9.5M): stay.
        assert_eq!(
            VolumeDecayTier::suggest_with_hysteresis(9_700_000, VolumeDecayTier::Tier2),
            VolumeDecayTier::Tier2
        );
        // Below the band: downgrade.
        assert_eq!(
            VolumeDecayTier::suggest_with_hysteresis(9_400_000, VolumeDecayTier::Tier2),
            VolumeDecayTier::Tier1
        );
    }

    #[test]
    fn hysteresis_moves_one_tier_at_a_time() {
        let v = 2_000_000_000;
        let t1 = VolumeDecayTier::suggest_with_hysteresis(v, VolumeDecayTier::Tier1);
        assert_eq!(t1, VolumeDecayTier::Tier2);
        let t2 = VolumeDecayTier::suggest_with_hysteresis(v, t1);
        assert_eq!(t2, VolumeDecayTier::Tier3);
        let t3 = VolumeDecayTier::suggest_with_hysteresis(v, t2);
        assert_eq!(t3, VolumeDecayTier::Tier4);
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

    #[derive(serde::Deserialize)]
    struct ConformanceFixture {
        #[allow(dead_code)]
        schema_version: u32,
        #[allow(dead_code)]
        description: String,
        cases: Vec<ConformanceCase>,
    }

    #[derive(serde::Deserialize)]
    struct ConformanceCase {
        id: String,
        trust_tier: TrustTier,
        rail: SettlementRail,
        amount_sat: u64,
        #[serde(default)]
        volume_decay_tier: VolumeDecayTier,
        #[serde(default = "default_system_load_factor")]
        system_load_factor: f64,
        #[serde(default)]
        enterprise_subscription_cap: bool,
        expected: ConformanceExpected,
    }

    #[derive(serde::Deserialize)]
    struct ConformanceExpected {
        percentage_fee_sat: u64,
        flat_floor_sat: u64,
        effective_fee_sat: u64,
        effective_bps: u32,
        distribution: FeeDistribution,
    }

    #[test]
    fn fee_conformance_vectors() {
        let fixture: ConformanceFixture =
            serde_json::from_str(include_str!("../fixtures/fee_conformance.json"))
                .expect("conformance fixture must parse");

        for case in fixture.cases {
            let result = calculate_dynamic_fee(FeeOptions {
                trust_tier: case.trust_tier,
                rail: case.rail,
                amount_sat: case.amount_sat,
                volume_decay_tier: case.volume_decay_tier,
                system_load_factor: case.system_load_factor,
                enterprise_subscription_cap: case.enterprise_subscription_cap,
            })
            .unwrap_or_else(|e| panic!("case {} rejected: {e}", case.id));

            assert_eq!(
                result.percentage_fee_sat, case.expected.percentage_fee_sat,
                "case {} percentage_fee_sat", case.id
            );
            assert_eq!(
                result.flat_floor_sat, case.expected.flat_floor_sat,
                "case {} flat_floor_sat", case.id
            );
            assert_eq!(
                result.effective_fee_sat, case.expected.effective_fee_sat,
                "case {} effective_fee_sat", case.id
            );
            assert_eq!(
                result.effective_bps, case.expected.effective_bps,
                "case {} effective_bps", case.id
            );
            assert_eq!(
                result.distribution, case.expected.distribution,
                "case {} distribution", case.id
            );
        }
    }
}
