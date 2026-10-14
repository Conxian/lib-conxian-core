# Core architecture

`lib-conxian-core` owns shared, deterministic protocol primitives for conxian.org. Commercial conxian-labs.com deployments consume these contracts; runtime provider connections, tenants, billing, storage, and custody belong to their services and the Enclave SDK.

Bitcoin L1 is the canonical trust/finality root for a universal-chain system. External networks, layers, assets, and independently evidenced protocol adapters remain in scope. There is no native Conxian token, tokenomics, or staking incentive.

| Boundary | Owner |
|---|---|
| Canonical chain taxonomy, trust tiers, risk, invariants | Core `control_model` |
| Exact operation/evidence declarations | Core `capability` |
| Proof/finality/request-response validation | Core `verifier` |
| Hardware signing, key isolation, provider attestation | Enclave SDK |
| Observation and state commitments | Nexus |
| Transport, financial-message translation, routing side effects | Gateway |
| Tenant deployment, monitoring, SLA orchestration | Platform / Business |

Known chain IDs, SDK re-export modules, runtime registrations, and production-qualified operations are distinct facts. The capability contract validates versions, exact chain/network/operation identity, freshness, architecture, and evidence consistency. It does not authenticate declarations or authorize value operations. See [capability contract](docs/CAPABILITY_CONTRACT.md), [existing boundaries](docs/ARCHITECTURE_BOUNDARIES.md), and [migration](docs/MIGRATION.md).

Existing public types and external-chain compatibility stay intact. The new contract is additive; downstream rollout requires approved exact dependency revisions and consumer policy/evidence verification.
