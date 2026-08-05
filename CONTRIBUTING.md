# Contributing to lib-conxian-core

## Architecture

`lib-conxian-core` is the protocol verification, trust-tier, and infrastructure
layer shared by all Conxian services. It is **transport-neutral**: core owns
deterministic contracts and types; network I/O lives in consumer crates.

### Module Map (17 modules)

| Module | Purpose | Consumer(s) |
|--------|---------|-------------|
| `control_model` | TrustTier (4 variants), Chain, BridgeSystem | Nexus, Gateway, Platform, SDK |
| `signing` | SignerCapabilities, SigningAlgorithm, SigningTarget | Nexus |
| `verifier` | 10+ protocol verification types | Nexus |
| `anchoring` | AnchoringPublisher, 8 types | Nexus |
| `bitcoin` | taproot, bip322 | Nexus |
| `protocol` | dlc, frost, covenant, intent | Nexus |
| `lightning` | LightningAdapter | Nexus |
| `adapters` | StateProofError | Nexus |
| `enclave` | AttestationCertificate, EnclaveVerificationError | Nexus |
| `contract_bridge` | typed ContractCall, DeploymentPlan | Gateway, Orbit |
| `babylon` | StakingIntent | Gateway |
| `fedimint` | FedimintMint | Gateway |
| `cjcs` | JobCard {context, type, work_intent} | Platform |
| `stacks` | SBTCBridge, Emily API | Gateway |
| `rgb` | GatewayRgbAdapter | Gateway |
| `chain` | ERC-7683 intent mapping, transport adapters | Gateway, Nexus |
| `crypto` | Key derivation | Internal |

### Feature Gates

```toml
[features]
default = []
full-sdk = ["dep:conxius-enclave-sdk"]
sdk-blockchain = ["full-sdk"]   # blockchain protocol types
sdk-signing = ["full-sdk"]      # signing primitives
sdk-cross-cutting = ["full-sdk"] # chain abstraction, ERC-7683
```

All SDK-dependent modules are gated behind `full-sdk`. Core types
(`control_model`, `signing`, `anchoring`) compile without the SDK.

### Testing

```bash
# Default feature set
cargo test --locked --all-targets

# All features
cargo test --locked --all-targets --all-features

# Specific module
cargo test --locked chain::transport
```

### Dependency Policy

- Zero new mandatory dependencies without architecture review
- `serde`, `thiserror`, `anyhow` for serialization/errors
- All SDK integration gated behind feature flags
- No network I/O in core — transport adapters live in consumers
