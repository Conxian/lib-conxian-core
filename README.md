# lib-conxian-core

Shared protocol primitives and reusable core libraries for the Conxian ecosystem.

## Purpose

Provide shared models, protocol primitives, and reusable core logic consumed by multiple Conxian services and public application surfaces.

## Status

**Active development (v0.1.0 baseline released).** This repository is a shared library layer and should be treated as reusable infrastructure rather than a top-level product surface.

## Scope

This repository contains shared core libraries and reusable primitives. It does not own public application UX, company administration, or private operational workflows.

## Governance relation

This repository is maintained by Conxian Labs as part of the public Conxian stack. It supports protocol and application layers while governance of the broader protocol evolves toward greater decentralization after mainnet.

## Relationship to the Conxian stack

- `Conxian` is the protocol core.
- `conxian-gateway` and `conxian-nexus` consume or align with shared infrastructure concerns.
- `conxius-wallet` and `conxian_ui` should rely on shared primitives here where cross-repo behavior belongs below the client layer.

## Development

```bash
cargo build
cargo test
```

## Security

Do not disclose vulnerabilities publicly. Use [SECURITY.md](./SECURITY.md) or `security@conxian-labs.com`.

## Policies

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [SECURITY.md](./SECURITY.md)
- [CODEOWNERS](./CODEOWNERS)
- [CHANGELOG.md](./CHANGELOG.md)
- [REPO_OWNERSHIP.md](./REPO_OWNERSHIP.md)
- [LICENSE](./LICENSE)

## Contact

- General: [info@conxian-labs.com](mailto:info@conxian-labs.com)
- Support: [support@conxian-labs.com](mailto:support@conxian-labs.com)
- Security: [security@conxian-labs.com](mailto:security@conxian-labs.com)

## License

See [LICENSE](./LICENSE).
