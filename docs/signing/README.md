# Signing guides

These guides describe the current per-chain signing boundaries in
`lib-conxian-core`. They are implementation-grounded documentation, not a
claim that Core contains production signers, transaction builders, RPC clients,
or provider workflows.

## Shared terminology

- **UCS** means the `UniversalChainSigner` contract in
  [`src/signing.rs`](../../src/signing.rs). A `SignRequest` carries an explicit
  `SigningTarget`, algorithm, `SigningPayload`, and derivation metadata. A
  `SignResponse` returns only public verification metadata; key custody remains
  in the concrete signer.
- **Capabilities are the source of truth.** A chain enum variant or a
  serializable request does not imply runtime support. `SignerCapabilities` must
  advertise the exact target, algorithm, operation, and address format before a
  signer can run.
- **Protocol verification is separate from signing.** The
  [`ProtocolVerifier`](../architecture/PROTOCOL_VERIFIER.md) façade validates
  capability, request, result, evidence binding, provenance, and finality
  metadata around a downstream backend. It does not acquire RPC evidence or
  prove cryptographic authenticity by itself.
- **Retry policy is downstream.** Core exposes typed failures and state models;
  Gateway decides whether a failed or pending operation is retried, waited on,
  or terminated.

## Guides

| Flow | Guide | Current Core anchor |
| --- | --- | --- |
| Bitcoin | [`bitcoin.md`](bitcoin.md) | BIP-341/BIP-342 handoff and BIP-110 size contract |
| Stacks / sBTC | [`stacks.md`](stacks.md) | `SBTCIntent`, `SBTCState`, and Bitcoin finality boundary |
| Babylon | [`babylon.md`](babylon.md) | BTC headers, EOTS evidence, and chain-identity mismatch |
| Liquid | [`liquid.md`](liquid.md) | Peg ownership and structural Elements checks |
| RGB | [`rgb.md`](rgb.md) | Transition/seal validation and Bitcoin anchoring |
| DLC | [`dlc.md`](dlc.md) | Funding/refund/CET/oracle ownership |

## Canonical contracts and ownership

- [Signing architecture](../SIGNING_ARCHITECTURE.md)
- [Architecture boundaries](../ARCHITECTURE_BOUNDARIES.md)
- [Protocol verifier source](../../src/verifier.rs)
- [BIP-110 alignment](../BIP110_ALIGNMENT.md)
- [BIP-110 types](../../src/control_model/bip110.rs)
- [Trust and chain-family mapping](../../src/control_model/trust.rs)

The production hardware-backed signer and attestation layer is the
[`conxius-enclave-sdk`](https://github.com/Conxian/conxius-enclave-sdk) contract
referenced by issue [#179](https://github.com/Conxian/conxius-enclave-sdk/issues/179).
Runtime orchestration and external effects belong to
[`conxian-gateway`](https://github.com/Conxian/conxian-gateway), with the flow
contract tracked in
[issue #245](https://github.com/Conxian/conxian-gateway/issues/245).
Observation and verifier backends belong to
[`conxian-nexus`](https://github.com/Conxian/conxian-nexus), tracked in
[issue #163](https://github.com/Conxian/conxian-nexus/issues/163). The
reference user-facing approval and fee-policy surface belongs to
[`conxius-wallet`](https://github.com/Conxian/conxius-wallet), tracked in
[issue #381](https://github.com/Conxian/conxius-wallet/issues/381).
