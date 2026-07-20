# Repo ownership

## Purpose

`lib-conxian-core` is the canonical home of shared capability interfaces and safety primitives for the Conxian builder platform.

## This repo owns

- canonical capability interfaces
- shared transaction intent models
- cross-layer safety and verification primitives
- signer policy abstractions
- shared data structures used by multiple layer adapters
- the versioned BIP-110 preflight request/result/error contract, including fixed-width byte
  measurements, phase/source provenance checks, deterministic findings, and the 257-byte
  Taproot control-block size boundary

## This repo does not own

- network adapters
- provider-specific integration logic
- wallet UX
- runtime orchestration
- consumer workflow logic
- transaction parsing, script classification, Taproot commitment/cryptographic validation, and
  downstream signing/broadcast enforcement

## Boundary rule

If code is specific to Bitcoin mainnet, Lightning, Stacks, Rootstock, Liquid, or a provider/runtime adapter, it should live outside this repo unless it is strictly required as a stable interface or shared primitive.

For CORE-005, Core defines only the serializable measurement contract and fail-closed size
findings. SDK and Wallet adapters own construction, serialization, parsing, and classification;
Gateway owns orchestration, persistence, routing, and external side effects. Downstream integration
is not implied by the presence of the Core API.

## Strategic role

Primary strategic repo.
