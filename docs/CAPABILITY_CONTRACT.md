# Universal capability contract

Core is shared public protocol infrastructure for conxian.org. Commercial deployments on conxian-labs.com consume these types; provider credentials, billing, tenants, network calls, and custody stay outside Core.

Bitcoin L1 is the canonical trust/finality root. The system serves external chains, networks, layers, and assets. Zero-token means no native Conxian token, tokenomics, or staking incentive; it does not remove independently evidenced external adapters.

`capability::known_chains()` derives identifiers and families from the existing canonical taxonomy. `cargo run --locked --example chain_taxonomy` exports it. A known identifier is not an implemented adapter or a production connection. SDK registry identifiers, asset records, protocol modules, and capability evidence rows are different inventories. Seventy evidence rows do not establish seventy live connections.

`CapabilityManifest` declares exact chain + network + operation tuples with repository/revision provenance, producer observation/expiry timestamps, runtime state, and separate API, implementation, integration, independent-review, and production evidence axes. It does not collapse chains into family aliases. Version 1 requires the Bitcoin root, universal-chain scope, no native token, bounded lifetime (at most 24 hours), no duplicate tuples, and qualified evidence for any production-supported declaration. Unknown JSON fields and unknown chains are rejected.

`validate_at(now)` performs deterministic structural validation only. It does not authenticate the producer, attest hardware, inspect evidence artifacts, or authorize signing. A consumer must bind the declaration to an authenticated provider/deployment, verify evidence and chain identity, enforce its policy, and recheck freshness before each protected operation. Unavailable or incompatible manifests must not gain support through registry fallback.

Observation, verification, routing, unsigned preparation, signing, broadcasting, recovery, and delegated execution are separate operations. A successful observation cannot authorize signing. Existing signing and verifier contracts remain authoritative for their domain.

Rollout is tracked in Core #440, Gateway #526, Nexus #418, Platform #1426, and Wallet #727. Downstream use requires an approved dependency revision and consumer validation; this additive contract does not upgrade those pins or claim a deployed negotiation flow.
