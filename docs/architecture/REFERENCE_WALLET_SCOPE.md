# Minimalist Reference Wallet Scope (CON-629)

This document defines the minimum feature set for the reference wallet, designed to prove the Vault SDK and security model without competing with downstream integrators.

## 1. Core Functional Requirements
- **SDK Handshake**: Visualizing and approving "Sovereign Handshakes" for agent-drafted intents.
- **Hardware-Backed Signing**: Direct integration with StrongBox TEE for secp256k1 and BIP327 MuSig2.
- **Policy Enforcement**: Local UI-level validation and display of enforced policies (Allowlists, Max Amount, Timelocks).
- **Mempool Integration**: Displaying current Bitcoin fee rates and congestion status for transaction planning.
- **L2 Telemetry**: Simple auditing of L2 block heights and finality status from the Gateway.

## 2. Technical Boundary
- **Zero-PII Persistence**: No personal information, analytics, or IP addresses are stored.
- **No Exchange Logic**: Centralized exchange integrations (Changelly, etc.) are excluded from the reference wallet.
- **Modular Adapters**: Uses the standard chain adapters provided by `lib-conxian-core`.

## 3. Exclusions (Integrator Surface)
- Multi-asset portfolio management beyond BTC and sBTC.
- Social recovery or managed custody patterns (relying entirely on sovereign hardware).
- Advanced DeFi yield farming dashboards (deferred to `Conxian_UI`).

## 4. Maintenance
The reference wallet codebase lives in `conxius-wallet` and is reviewed against this scope monthly.
