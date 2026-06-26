# Conxian Improvement Proposals (CXIP) Index

This index tracks the formal improvement proposals for the Conxian protocol and ecosystem.

## Active Proposals

| ID | Title | Status | Summary |
| :--- | :--- | :--- | :--- |
| **CXIP-20** | **Modular Protocol Architecture** | **Implemented** | Defines the separation of Vault SDK, Gateway, and Enclave surfaces. |
| **CXIP-21** | **Universal Chain Adapter Standard** | **Implemented** | Standardizes the interface for multi-chain (EVM, Bitcoin, Cosmos) support. |
| **CXIP-22** | **Trust-Tier Policy Enforcement** | **Implemented** | Formalizes T1-T4 trust classification for bridge and messaging lanes. |
| **CXIP-23** | **Agentic MCP Surface** | **Implemented** | Defines the Model Context Protocol (MCP) toolset for autonomous interaction. |
| **CXIP-26** | **Cross-Chain Intent Solving (ERC-7683)** | **Implemented** | Implements the competitive solver selection and bidding algorithm. |
| **CXIP-27** | **Threshold Signature Infrastructure (FROST)** | **Implemented** | Implements the FROST TSS manager for multi-party signing. |
| **CXIP-28** | **Bitcoin Recursive Covenants (OP_CAT)** | **Implemented** | Defines the template library for OP_CAT-based vault covenants. |

## Research & Exploration

- **CXIP-24**: Verifiable Intent Discovery (Researching)
- **CXIP-25**: BitVM2 Optimistic Bridge Finality (Researching)

## Governance Process

Proposals move through the following lifecycle:
1. **Draft**: Initial research and specification.
2. **Review**: Feedback from core maintainers and partners.
3. **Approved**: Finalized specification ready for implementation.
4. **Implemented**: Code merged and verified in `lib-conxian-core` or `conxian-gateway`.
5. **Deprecated**: Superseded by a newer proposal.

Refer to [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on how to submit a new CXIP.
