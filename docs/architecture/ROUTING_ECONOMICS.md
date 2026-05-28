# Routing-Fee Economics & Failure Modes (CON-631)

This document analyzes the sustainability of embedded routing and execution fees as a business model for the Conxian Vault SDK.

## 1. Economic Model
- **Mechanism**: Small percentage or fixed satoshi fee applied to cross-chain maneuvers and yield-rebalancing intents drafted by the SDK.
- **Participation**: Fees are split between the Protocol Reserve (5%), Labs Operations (5%), and Contributor Rewards (90% stream).

## 2. Failure Modes & Risks

### 2.1 Margin Pressure
- **Risk**: Integrators may fork the SDK to remove fee hooks or switch to zero-fee alternative routing providers.
- **Mitigation**: Focus on "Unique Value Add" such as TEE-verified proofs and complex maneuver orchestration that is difficult to replicate.

### 2.2 Adoption Barriers
- **Risk**: Institutional partners may reject embedded fees for compliance or predictable-cost reasons.
- **Mitigation**: Provide a "flat-fee" or "license-based" alternative for high-volume enterprise integrators.

### 2.3 Systemic Failure
- **Risk**: A major exploit on a supported L2 (e.g., BOB or Hemi) could result in lost funds, destroying the SDK's reputation and routing volume.
- **Mitigation**: Maintain a fail-closed status and mandatory timelocks on all state proposals.

## 3. 24-Month Outlook
Sustainability depends on high-volume, low-friction integration. If the Conxian SDK becomes the "Stripe for Bitcoin Signing," routing fees will support a durable contributor ecosystem.

## 4. Maintenance
Economic models are reviewed during the quarterly Strategy Alignment review (CON-548).
