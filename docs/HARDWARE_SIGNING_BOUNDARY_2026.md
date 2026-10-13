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
