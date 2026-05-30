# lib-conxian-core

![CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)

Shared core libraries and reusable primitives for the Conxian ecosystem.

## Purpose

Centralize shared models, APIs, and core logic used by Conxian Gateway and downstream consumers such as platform services, wallet integrations, and tooling.

## Status

Active development.

## Scope

This repository contains shared technical primitives and common logic. It should not contain company administrative systems, private strategic records, or unrelated product-specific UX logic.

## Governance relation

This repository is maintained by Conxian Labs as shared infrastructure supporting public Conxian services and applications.

## Audience

- gateway engineers
- platform developers
- wallet and integration contributors
- maintainers working on shared models and observability

## Relationship to the Conxian stack

- consumed by `conxian-gateway`
- used by platform services and integrations
- shared across multiple public repositories where logic should not be duplicated

## Security

This repository is security-sensitive shared infrastructure. Use [SECURITY.md](SECURITY.md) for reporting guidance.

## Release hygiene

- semantic versioning
- changelog-based releases
- dual licensing under MIT and Apache 2.0

## License

Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE)
