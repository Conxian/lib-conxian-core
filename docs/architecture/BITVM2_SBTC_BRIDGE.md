# Conxian Target Architecture: BitVM2 & sBTC Bridge

## 1. Overview
The Conxian network utilizes a trust-minimized bridging model anchored to Bitcoin. This architecture combines the optimistic verification of BitVM2 with the threshold-security model of Stacks sBTC to ensure protocol solvency and user sovereignty.

## 2. BitVM2 Optimistic Verification
BitVM2 allows for the verification of complex cryptographic proofs (Groth16 SNARKs) directly on Bitcoin without a soft fork.

### 2.1. Verification Segments
To fit within Bitcoin's script limits, verification is split into 364 independent segments:
- **1 Validating Tap:** Core arithmetic verification of the SNARK.
- **363 Hashing Taps:** Hash chain verification for intermediate states.

### 2.2. Disprove Mechanism
If an operator provides an invalid state root, verifiers can broadcast a **disprove transaction**.
- **Logic:** Compares computed output hashes against operator-claimed hashes.
- **Fail-Closed:** If the input is invalid or the output hash mismatch, the disprove transaction succeeds, and the operator is slashed or the transition is reverted.

## 3. sBTC Peg Orchestration
sBTC provides a decentralized peg for Bitcoin on the Stacks layer, utilizing a 70% threshold signer set.

### 3.1. Peg-In (BTC -> sBTC)
1. User sends BTC to the threshold peg-wallet.
2. Signers verify the transaction and update the Stacks state.
3. sBTC is minted to the user's Stacks address.

### 3.2. Peg-Out (sBTC -> BTC)
1. User initiates withdrawal via the Clarity `sbtc-registry` contract.
2. Request is marked as **PENDING**.
3. Signers verify and decide to **ACCEPT** or **REJECT**.
4. If accepted, BTC is sent from the peg-wallet, and sBTC is burned.
5. Status is updated to **CONFIRMED**.

## 4. Bridge Orchestration Logic
The Conxian Gateway (standalone) acts as the primary orchestrator for these flows, monitoring both Bitcoin and Stacks chains.

### 4.1. Intent Alignment
External settlement triggers (ISO 20022, etc.) generate state proposals that are subject to a 144-block timelock, matching the BitVM2 challenge period and sBTC finality requirements.

### 4.2. TEE Guardian Role
The StrongBox TEE performs real-time verification of bridge state and challenge monitoring, ensuring that the Guardian attestation remains valid only if no active BitVM challenges are pending or successful.

## 5. Mainnet Readiness Standards
- **Zero Mocks:** All production paths must use real chain data or TEE-verified state.
- **Fail-Closed:** Any ambiguity in bridge state results in a temporary halt of execution (Emergency Mode).
- **Audit Trails:** All bridge transactions and proposals are logged in the decentralized Tableland persistence layer.
