# Enhancement Roadmap: lib-conxian-core Protocol and SDK Alignment

## 1. Current ownership baseline

The standalone Conxian Gateway owns runtime orchestration, live risk inputs,
provider selection, persistence, and external side effects. `lib-conxian-core`
owns canonical protocol primitives, state/control contracts, and invariant
validation without network or provider behavior.

Production hardware-backed signing, attestation, and policy flows belong to the
canonical [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk).
The workspace `lib-conxian-core-enclave` crate is a narrow compatibility adapter
for mapping Core contracts to the exact SDK API; it is not a provider or runtime
SDK.

The phase records below are historical roadmap and downstream-integration notes.
They are not claims that Core owns production Vault SDK, MuSig2, BitVM2,
hardware/provider, or runtime implementations.

## 2. Phase 7: Advanced Risk Metrics & Expanded Layer Support (Historical)

- **Historical record:** Earlier Gateway-oriented work described a multi-factor
  risk engine, high-precision TVL aggregation, and expanded layer coverage.
- **Current boundary:** Core retains the typed risk-profile artifact, score
  bounds, provenance rules, and trust-tier invariants; live metrics and routing
  remain Gateway/Nexus responsibilities.
- **Protocol coverage:** Core provides canonical contracts and fail-closed
  structural validation. Production BitVM2 verification and signing flows are
  SDK-owned rather than Core runtime capabilities.

## 3. Phase 8: Mainnet Node Integration & Direct Bridges (Historical downstream record)

- **Historical objective:** Move downstream monitoring from simulated data toward
  authenticated mainnet integrations.
- **Ownership correction:** Hiro/Bitcoin/sidechain RPC access, reserve
  observation, mempool analysis, and finality tracking belong to Gateway, Nexus,
  Wallet, or another audited downstream provider.
- **Signing boundary:** Production BIP-327 MuSig2 sessions and hardware-backed
  signing belong to `conxius-enclave-sdk`; Core exposes only the protocol-facing
  contracts and invariants consumed by those flows.
- **Bridge orchestration:** BitVM challenge handling, segment orchestration, and
  external settlement triggers are downstream runtime workflows, not Core-owned
  provider implementations.

## 4. Phase 9: Agentic Surface & Autonomous Systems (Historical downstream record)

- **Historical objective:** Enable programmatic trust and autonomous interaction
  through Model Context Protocol (MCP) surfaces.
- **Ownership correction:** MCP/REST endpoints, background intent broadcasting,
  proposal execution, and human-in-the-loop workflow coordination belong to the
  standalone Gateway or platform layer.
- **Next steps:** Keep the Core contract stable while downstream consumers use
  `conxius-enclave-sdk` through `lib-conxian-core-enclave` for hardware-anchored
  execution of approved intents. Expand MCP resources only in the owning runtime.

## 2026-06-26 Hardening Pass (Historical)

- Aligned the historical documentation and code-versioning record to **v0.2.10**.
- Recorded integration paths for BitVMX, BitVM3, and ZKCP as research or
  downstream work, not Core runtime ownership.
- Updated scorecards to reflect protocol-boundary stability.
- Hardened CI/CD workflow pins and verified hygiene standards.
