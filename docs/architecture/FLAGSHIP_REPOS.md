# Flagship Repository Selection & Narrative Order (CON-298)

This document defines the selection and ordering of pinned flagship repositories for the Conxian GitHub organization. This set is designed to present a coherent story of Conxian as a provider of native Bitcoin application infrastructure.

## 1. Narrative Order (Pinned Repos)

The following repositories should be pinned in the GitHub organization in this exact order:

1.  **`lib-conxian-core` (The Vault SDK)**
    - **Role**: Primary commercial primitive and cryptographic foundation.
    - **Narrative**: "Start here. Secure signing and policy enforcement for Bitcoin apps."
    - **Pinned Description**: Production-grade Vault SDK and protocol primitives for native Bitcoin applications. Hardware-backed signing, BIP327 MuSig2, and BitVM2.

2.  **`conxius-wallet` (The Reference Client)**
    - **Role**: Proof-of-concept application and reference integration.
    - **Narrative**: "The SDK in action. See how a sovereign Bitcoin wallet is built."
    - **Pinned Description**: Reference asset management client proving the Vault SDK and StrongBox TEE security model. Sovereign, biometric, and audit-ready.

3.  **`conxian-gateway` (The Protocol Router)**
    - **Role**: Connectivity and telemetry layer for Bitcoin L2s and sidechains.
    - **Narrative**: "Scale and connect. Unified access to the entire Bitcoin ecosystem."
    - **Pinned Description**: Unified Gateway for Conxian Network sovereign services and Bitcoin Layers. Real-time telemetry, risk metrics, and Agentic MCP surface.

4.  **`conxius-platform` (The Orchestration Engine)**
    - **Role**: Industrial-grade automation and institutional settlement triggers.
    - **Narrative**: "Automate and settle. Bridging native Bitcoin to institutional rails."
    - **Pinned Description**: Workflow orchestration and institutional settlement logic for the Conxian ecosystem. ISO 20022 and PAPSS integration.

5.  **`Conxian_UI` (The Interface Surface)**
    - **Role**: Visual dashboards and product landing sites.
    - **Narrative**: "See the metrics. High-transparency interfaces for protocol trust."
    - **Pinned Description**: Product dashboards and landing surfaces for the Conxian ecosystem. High-contrast, responsive, and type-safe.

## 2. Classification Summary

| Repo | Taxonomy | Public Status |
| :--- | :--- | :--- |
| `lib-conxian-core` | SDK & Core | **Flagship (Pinned #1)** |
| `conxius-wallet` | Reference Client | **Flagship (Pinned #2)** |
| `conxian-gateway` | Supporting Infra | **Flagship (Pinned #3)** |
| `conxius-platform` | Shared Runtime | **Flagship (Pinned #4)** |
| `Conxian_UI` | Product UI | **Flagship (Pinned #5)** |
| `conxian-business` | Governance & OS | **Internal/Strategic (Not Pinned)** |

## 3. Pinned Description Strategy

- Descriptions must start with a punchy value proposition.
- Technical buzzwords (MuSig2, BitVM2, TEE) are included for developer credibility but subordinated to business utility.
- All descriptions must align with the "Native Bitcoin Application Infrastructure" positioning (CON-632).

## 4. Maintenance

This selection is reviewed during the weekly GTM Review (CON-243). Any repository role shift or new product launch requires a re-evaluation of this list.
