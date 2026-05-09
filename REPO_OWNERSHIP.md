# Repo ownership

## Purpose

`lib-conxian-core` is the canonical home of shared capability interfaces and safety primitives for the Conxian builder platform.

## This repo owns

- canonical capability interfaces
- shared transaction intent models
- cross-layer safety and verification primitives
- signer policy abstractions
- shared data structures used by multiple layer adapters

## This repo does not own

- network adapters
- provider-specific integration logic
- wallet UX
- runtime orchestration
- consumer workflow logic

## Boundary rule

If code is specific to Bitcoin mainnet, Lightning, Stacks, Rootstock, Liquid, or a provider/runtime adapter, it should live outside this repo unless it is strictly required as a stable interface or shared primitive.

## Strategic role

Primary strategic repo.