# Repository Ownership & Governance Boundaries

This document defines the ownership boundaries, classification, and governance structure for `lib-conxian-core` within the Conxian ecosystem.

## Classification

`lib-conxian-core` is classified as **Primary Strategic Platform Core (Tier 1)**.

## Ownership Scope

`lib-conxian-core` owns:
- Core cryptographic primitives (MuSig2, FROST, BIP-340 Taproot, BIP-352 Silent Payments, PVDE, Adaptor Signatures)
- Protocol models and intent serialization (FDC3, DLC, OP_CAT covenants, BitVM2/3, Fedimint ECC blinding, Stacks Nakamoto/sBTC, Liquid sidechain adapters, CJCS job cards, RGB stock contracts)
- Verification facades and fail-closed validation engines
- Universal chain adapter DTOs and transport capabilities

Downstream entities (such as `conxian-gateway`, `conxius-enclave-sdk`, `conxius-wallet`) depend on `lib-conxian-core` as a core library.

## Support & SLA Expectations

- **Public Repository (Open Source):** No runtime, uptime, consensus finality, or protocol execution SLAs are provided for public code. Support is community-best-effort with target 48-hour response windows for non-security issues and 48h/5-day triage SLAs for security disclosures per [SUPPORT.md](../../SUPPORT.md) and [SECURITY.md](../../SECURITY.md).
- **Enterprise Commercial Tier:** Legally binding commercial SLAs apply strictly to signed B2B commercial contracts covering enterprise middleware (`conxian-gateway`) and integration wrappers.

## Maintainer Paths & CODEOWNERS

Critical architectural paths and release documentation are protected under [.github/CODEOWNERS](../../.github/CODEOWNERS).
