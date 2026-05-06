# SAB Migration Timeline & Cutover Plan (CON-332)

This document defines the execution timeline, migration waves, and rollback criteria for the Sovereign Asset Bridge (SAB) and the broader Conxian ecosystem transition to mainnet-ready operation.

## 1. Migration Waves

### Wave 1: Foundation & Hygiene (Current)
**Focus**: Repository security, ZSE compliance, and SDK-first positioning.
- **Tasks**:
  - Remediate tracked secrets and .env exposure (CON-544).
  - Standardize governance files (README, SECURITY, LICENSE) (CON-432).
  - Pivot company positioning to "Native Bitcoin Apps" (CON-632).
- **Gate**: 100% CI pass rate and clean hygiene scan on all P0 repos.
- **Rollback**: Local branch revert; no on-chain impact.

### Wave 2: Vault SDK & Partner Integration (T + 15 Days)
**Focus**: Commercialization of the first SDK primitive and partner validation.
- **Tasks**:
  - Release Vault SDK v0.2.x with BIP327 MuSig2 (CON-633).
  - Pilot SDK with internal reference wallet (conxius-wallet).
  - Complete partner intake workflow definition (CON-244).
- **Gate**: Successful hardware-backed signature verification in reference client.
- **Rollback**: Decommission SDK version; revert to internal-only protocol tests.

### Wave 3: Gateway Mainnet Connectivity (T + 30 Days)
**Focus**: Transition from simulated telemetry to real-time production node RPCs.
- **Tasks**:
  - Connect Gateway to Bitcoin Core and Core DAO mainnet nodes (Phase 8).
  - Enable automated mempool analysis and congestion triggers.
  - Activate model-carrying telemetry via MCP (Phase 9).
- **Gate**: "Verified (On-chain)" status reflected for all supported L2/Sidechain reserves.
- **Rollback**: Fail-closed transition to "ConnectionRequired" state; revert to last known good telemetry seeds.

### Wave 4: Full Orchestration & Institutional Cutover (T + 45 Days)
**Focus**: Execution of external settlement triggers and SAB-owned treasury handoff.
- **Tasks**:
  - Map SAB-owned wallet destinations and handoff sequence (CON-482).
  - Deploy TEE-verified ISO 20022 and PAPSS intent triggers (CON-162).
  - Final GTM go/no-go review (CON-243).
- **Gate**: 144-block timelock enforcement verified on a live mainnet proposal.
- **Rollback**: Point-in-time restore of GCP infrastructure; trigger emergency manual override for inflight timelocks.

## 2. Dependency Map
- **Hard Blocker**: GitHub billing lock resolution (Non-code platform constraint).
- **Critical Path**: Vault SDK hardening -> Gateway RPC integration -> Institutional handoff.

## 3. Rollback Strategy
- **Logic**: All state changes are proposal-first. Rollback involves discarding the pending state proposal before execution.
- **Infrastructure**: Blue-green deployment on GCP allows immediate traffic redirection to the previous stable binary.
- **Data**: Database migrations follow the 'pre-applied' and 'idempotent' patterns defined in HYGIENE_BASELINE.md.

## 4. Maintenance
This plan is reviewed during the weekly Operating Loop (CON-224).
