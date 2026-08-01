# Conxian Improvement Proposals (CXIP) Index

This index tracks the formal improvement proposals for the Conxian protocol and ecosystem.

Proposal status describes governance/specification state, not proof that Core
contains a production verifier. Current `lib-conxian-core` boundaries are:

- FROST share generation, distribution, and aggregation are unsupported until
  an audited implementation is supplied; production FROST is SDK-owned.
- Core enclave DER parsing is not certificate-chain or hardware-attestation
  verification; production hardware attestation belongs in
  `conxius-enclave-sdk`.
- Core BIP-322 handling is limited to address/base64/witness shape parsing, not
  cryptographic script or signature verification.
- Fedimint point reconstruction is a deterministic primitive, not
  provider-backed mint/note/status verification; authenticated status is
  unavailable without a provider.

These are the current v0.3.1 Core boundaries. Proposal status below describes
governance/specification state only and does not authorize a production flow;
the verifier inventory is the source of truth for evidence and compatibility
wrappers.

## Active Proposals

| ID | Title | Status | Summary |
| :--- | :--- | :--- | :--- |
| **CXIP-20** | **Modular Protocol Architecture** | **Implemented** | Defines the separation of Vault SDK, Gateway, and Enclave surfaces. |
| **CXIP-21** | **Universal Chain Adapter Standard** | **Implemented** | Standardizes the interface for multi-chain (EVM, Bitcoin, Cosmos) support. |
| **CXIP-22** | **Trust-Tier Policy Enforcement** | **Implemented** | Formalizes T1-T4 trust classification for bridge and messaging lanes. |
| **CXIP-23** | **Agentic MCP Surface** | **Implemented** | Defines the Model Context Protocol (MCP) toolset for autonomous interaction. |
| **CXIP-26** | **Cross-Chain Intent Solving (ERC-7683)** | **Implemented** | Implements the competitive solver selection and bidding algorithm. |
| **CXIP-27** | **Threshold Signature Infrastructure (FROST)** | **Implemented** | Defines the FROST TSS boundary; Core share/distribution/aggregation remains unsupported until an audited implementation is supplied, with production implementation SDK-owned. |
| **CXIP-28** | **Bitcoin Recursive Covenants (OP_CAT)** | **Implemented** | Defines the template library for OP_CAT-based vault covenants. |
| **CXIP-29** | **MuSig2 Signature Aggregation** | **Implemented** | Formalizes BIP-327 signature aggregation for efficient multi-sig. |
| **CXIP-30** | **DLC Native Finance Primitives** | **Implemented** | Maps Discreet Log Contracts (DLC) to the Universal Settlement Interface. |
| **CXIP-32** | **Hardware Attestation (X.509 DER)** | **Implemented** | Defines Core DER container parsing; certificate-chain and hardware-attestation verification is SDK-owned. |

## Research & Exploration

- **CXIP-24**: Verifiable Intent Discovery (Researching)
- **CXIP-25**: BitVM2 Optimistic Bridge Finality (Researching)
- **CXIP-31**: BitVMX Adaptive Proofs (Researching)
- **CXIP-33**: ZKCP Atomic Settlement (Draft)

## Governance Process

Proposals move through the following lifecycle:
1. **Draft**: Initial research and specification.
2. **Review**: Feedback from core maintainers and partners.
3. **Approved**: Finalized specification ready for implementation.
4. **Implemented**: Code merged and verified in `lib-conxian-core` or `conxian-gateway`.
5. **Deprecated**: Superseded by a newer proposal.

Refer to [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on how to submit a new CXIP.
