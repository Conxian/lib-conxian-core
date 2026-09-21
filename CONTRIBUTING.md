# Contributing to lib-conxian-core

Thank you for your interest in contributing to `lib-conxian-core`.

This document provides guidelines and standards for contributing to the core protocol library of the Conxian ecosystem. Following these process rules helps ensure quality, security, architectural compliance, and smooth code reviews.

---

## Architecture & Scope

`lib-conxian-core` provides protocol verification, trust-tier models, and shared infrastructure primitives for all Conxian ecosystem services.

- **Transport-Neutral:** Core owns deterministic state machines, contracts, and types. Network I/O and runtime side effects live in consumer services (e.g., `conxian-gateway`, `conxian-nexus`).
- **Zero Secret Egress (ZSE):** No environment variable readers, secret leaks, or insecure default private key initializers exist in core production code.

### Module Map (19 modules)

| Module | Purpose | Consumer(s) |
|--------|---------|-------------|
| `control_model` | TrustTier (4 variants), Chain (48 variants), ChainFamily (17 variants), Bip110Compliance, BridgeSystem (28 variants), ProofEnvelope, RailMetadata, WalletAuthority, ControlModelAdapter trait, CanonicalRiskProfileSet, Bip110PreflightValidator | Nexus, Gateway, Platform, SDK |
| `signing` | SigningAlgorithm, DigestAlgorithm, SignatureEncoding, AddressFormat (21 variants), SigningOperation, SigningPayload, SignRequest, SignResponse, UniversalChainSigner trait, SignerCapabilities | Nexus |
| `verifier` | 30+ protocol verification types (ProtocolVerifier, ProofVerificationRequest/Result, VerifierCapabilities, TransactionFinalityStatus, etc.) | Nexus |
| `anchoring` | AnchoringPublisher, 8 types | Nexus |
| `bitcoin` | Taproot, BIP-322, BIP-352 Silent Payments | Nexus |
| `protocol` | DLC, FROST, Covenant (OP_CAT), Intent | Nexus |
| `lightning` | LightningAdapter, BOLT-12, BIP-353 | Nexus |
| `adapters` | StateProofError | Nexus |
| `enclave` | AttestationCertificate, EnclaveVerificationError | Nexus |
| `contract_bridge` | Typed ContractCall, DeploymentPlan | Gateway, Orbit |
| `babylon` | StakingIntent | Gateway |
| `fedimint` | FedimintMint | Gateway |
| `cjcs` | CjcsError, WorkIntent, JobCard (JSON-LD, fail-closed validation) | Platform |
| `stacks` | SBTCBridge, Emily API | Gateway |
| `rgb` | GatewayRgbAdapter | Gateway |
| `chain` | ERC-7683 intent mapping, transport adapters | Gateway, Nexus |
| `crypto` | CryptoError, PVDE, WitnessEncryption, AdaptorSignature | Internal |
| `deployment` | DeploymentPlan, contract deployment manifests | Internal |
| `sdk` | 74 SDK module re-exports (feature-gated) | Nexus, Gateway |

---

## Architectural Boundary Enforcement (CON-700)

Core enforces strict architectural boundaries to guarantee pure, deterministic verification behavior:

1. **Forbidden I/O Imports:** Production code in `src/` must NOT import or use `std::net`, `std::fs`, `std::process`, or `std::env`.
2. **Feature Gating:** SDK integration must remain gated under the `full-sdk` or specific `sdk-*` feature flags. Core default features must compile cleanly without external SDK dependencies.
3. **Fail-Closed Validation:** All parameter validators (such as `WorkIntent::validate`, `SBTCIntent::validate`, `LiquidPegIntent::validate`) must reject invalid, empty, or uncanonical inputs deterministically before execution.

---

## Feature Gates

```toml
[features]
default = []
full-sdk = ["dep:conxius-enclave-sdk"]
sdk-blockchain = ["full-sdk"]   # blockchain protocol types
sdk-signing = ["full-sdk"]      # signing primitives
sdk-cross-cutting = ["full-sdk"] # chain abstraction, ERC-7683
```

Core types (`control_model`, `signing`, `anchoring`, etc.) compile without the SDK.

---

## Contribution Workflow

### 1. Branching & Issue Tracking
- Create a feature branch off `main` with a descriptive name, e.g., `feature/con-123-hardened-adapter` or `fix/con-456-covenant-invariant`.
- Link your pull request to the relevant tracking issue (e.g., `Closes #123` or `CON-123`).

### 2. Commit Message Standards
Use clear, imperative commit messages adhering to standard conventions:
- Keep the subject line under 50 characters.
- Capitalize the subject line and do not end with a period.
- Format: `<type>(<scope>): <short description>` (e.g., `feat(cjcs): add fail-closed validation for JobCard`).

### 3. Local Verification Commands
Before opening a Pull Request, you MUST run and pass all local verification checks:

```bash
# 1. Format and Linter checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings

# 2. Rust Workspace Test Suite
cargo test --workspace --locked
cargo test --workspace --all-features --locked

# 3. Architectural Boundary & Hygiene Guard Verification
python3 scripts/verify_contamination_guard.py
python3 scripts/verify_tracked_artifacts.py
python3 scripts/verify_release_version.py --phase source-only
python3 scripts/verify_release_hygiene.py

# 4. Repository Verification Test Suite
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
```

---

## Security & CODEOWNERS Reviews

- **Zero Secrets:** Never commit API keys, private keys, environment files (`.env`), or certificates.
- **Sensitive Files:** Changes to `CODEOWNERS`, `SECURITY.md`, `SUPPORT.md`, `.github/workflows/`, and `release.yml` require explicit review from maintainers specified in [.github/CODEOWNERS](.github/CODEOWNERS).
- **Vulnerability Reporting:** Do NOT submit public issues for security vulnerabilities. Follow the private reporting instructions in [SECURITY.md](SECURITY.md).

---

## Pull Request Checklist

When submitting a PR, ensure that:
- [ ] Code builds without warnings on Rust `1.98.1+`.
- [ ] All Rust unit, doc, and integration tests pass cleanly (`cargo test --workspace --all-features`).
- [ ] All Python verification guard scripts and unit tests pass.
- [ ] Documentation (`README.md`, `CHANGELOG.md`, module rustdocs) is updated to reflect changes.
- [ ] PR template checklist items are completed.
