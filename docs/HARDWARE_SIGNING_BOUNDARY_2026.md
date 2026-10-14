# Hardware Signing Boundary (2026)

`lib-conxian-core` is the provider-neutral protocol/type layer. Hardware
drivers, raw attestation parsing, vendor collateral, key custody, and provider
qualification belong in `conxius-enclave-sdk` or a commercial deployment
adapter—not in Core.

## Non-negotiable boundary

- Bitcoin L1 is the native anchor. Core introduces no native token, staking,
  or tokenomics.
- Core may accept only the SDK's typed, provider-verified capability and a
  response bound to the exact operation, key, algorithm, purpose, policy, and
  replay identity.
- Core must not select trust by chipset/SKU, module label, version string,
  cloud name, environment variable, or self-reported `hardware=true` flag.
- Mock, simulator, POC, SoftHSM, sample JSON, and structurally valid evidence
  never satisfy a value-bearing hardware requirement.
- Unsupported or unavailable providers fail closed; there is no software
  fallback for value-bearing operations.

## Hardware Chipset & Driver Abstraction in Core

Core provides strongly-typed abstractions in `src/enclave/mod.rs` covering all 11 target hardware environments:
- **`HardwareChipset`**: `AwsNitro`, `AndroidStrongBox`, `IntelSgx`, `IntelTdx`, `AmdSevSnp`, `AppleSecureEnclave`, `Pkcs11Hsm`, `RiscVKeystone`, `Tpm20`, `ArmTrustZone`, `HardwareWallet`.
- **`AttestationType`**: `NitroCoseSign1`, `KeyMintX509Chain`, `IntelDcapQuote`, `AmdSnpReport`, `AppleAppAttest`, `Pkcs11SlotAttestation`, `KeystoneReport`, `Tpm2Quote`, `OpTeeAttestation`, `SecureElementCert`.
- **`ChipsetDriverInfo`**: Validates system device nodes (`/dev/*`, `hal:*`, `sys:*`, `/usr/lib/*`), driver versions, and vendor IDs.
- **`HardwareAttestationEvidence`**: Validates 32-byte non-zero challenge nonces, measurement digests (minimum 32 bytes), raw payloads, and chipset-attestation compatibility before returning `UnsupportedProvider` for cryptographic root certificate verification (owned by `conxius-enclave-sdk`).

For the comprehensive technical specification covering all chipsets, kernel drivers, HAL APIs, attestation certificate roots, and physical attack mitigations, see [`docs/signing/HARDWARE_CHIPSET_DRIVERS_AND_ENCLAVE_RESEARCH.md`](signing/HARDWARE_CHIPSET_DRIVERS_AND_ENCLAVE_RESEARCH.md).

## Provider ownership

| Concern | Owner |
|---|---|
| Canonical operation and response types | Core + SDK contract |
| Android KeyMint/StrongBox, Nitro, PKCS#11/HSM, TPM, DCAP, SEV-SNP, Apple, cloud HSM, Keystone adapters | SDK/provider adapters |
| Roots, certificate paths, revocation, collateral, TCB and measurement policy | SDK/provider verifier |
| Durable multi-replica replay and audit evidence | Managed service/runtime |
| Real-device/provider tests, SBOM, provenance and independent review | Release qualification |

The authoritative provider matrix and HS-001…HS-013 backlog are maintained in
`conxius-enclave-sdk/docs/architecture/HARDWARE_SIGNING_PROVIDER_MATRIX_2026.md`.
Until a provider row is promoted with exact evidence, Core integrations must
treat it as unsupported.
