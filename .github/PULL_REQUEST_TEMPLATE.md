## Description
<!-- Provide a brief summary of the changes and the Linear issue being addressed. -->

## Linked Issues
- Fixes [CON-XXX]

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Security hardening / alignment

## BOS Classification (CON-412)
- [ ] **Docs-only**: Documentation, README, or strategic spec changes.
- [ ] **Stub-isolation**: Refactoring to move mocks/stubs out of production paths.
- [ ] **Dev-only implementation**: Feature work targeting the `dev` branch only.
- [ ] **Production implementation**: Core logic targeting `main` (must be mainnet-ready).

## Mandatory Checklist
- [ ] My code follows the style guidelines of this project (cargo fmt).
- [ ] I have performed a self-review of my own code.
- [ ] I have commented my code, particularly in hard-to-understand areas.
- [ ] I have made corresponding changes to the documentation.
- [ ] My changes generate no new warnings (cargo clippy).
- [ ] I have added tests that prove my fix is effective or that my feature works.
- [ ] New and existing unit tests pass locally with my changes.
- [ ] Any public/private boundaries are respected (ZSE compliant).

## Mainnet Safety & Release Operations
- [ ] This PR contains NO mocks, stubs, or testnet-only logic for the `main` branch.
- [ ] Intent-based actions follow the StateProposal/Timelock flow (CON-162).
- [ ] Verified that UI, docs, and config match actual runtime behavior.
- [ ] Required status checks and linting have passed.
