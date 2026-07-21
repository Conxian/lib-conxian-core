# Taproot, Tapscript, and Miniscript support boundary

**Status:** Core structural contract, API version `1`
**Tracking:** [CON-1501](https://linear.app/conxian-labs/issue/CON-1501/core-002-validate-bip-341bip-342-and-miniscript-invariants), [GitHub #178](https://github.com/Conxian/lib-conxian-core/issues/178)

## Scope and claims boundary

`lib-conxian-core` provides neutral, auditable contracts for the byte shapes and public metadata
that a transaction-aware adapter can hand across the Core/SDK boundary. The implementation is in
[`src/bitcoin/taproot.rs`](../../src/bitcoin/taproot.rs) and is re-exported from
[`src/bitcoin/mod.rs`](../../src/bitcoin/mod.rs).

Successful results carry the explicit `structural_only` claim. Core does **not** claim to:

- parse or construct transactions, ScriptPubKeys, descriptors, or PSBTs;
- verify a Schnorr signature, secp256k1 point, Taproot tweak, Merkle commitment, or control-block
  path;
- select a BIP-341 sighash, manage keys, sign, or integrate hardware custody;
- execute Tapscript or enforce `OP_SUCCESSx`, `MINIMALIF`, `CHECKSIGADD`, disabled opcodes, sigops,
  stack, or other runtime/resource rules;
- parse, compile, optimize, satisfy, or execute Miniscript; or
- decide activation, deployment, UTXO history, fee policy, broadcast, persistence, or RPC behavior.

The caller must treat `structural_only` as **not cryptographically verified** and **not runtime
executed**. Errors are category/code pairs and do not include raw transaction, script, signature,
control-block, key, or secret bytes.

## Core API

| API | Core checks | Resulting claim | Not checked here |
| --- | --- | --- | --- |
| `validate_p2tr_witness_program` | Witness version is `1`; witness program is exactly 32 bytes. | P2TR witness-program shape. | Whether the 32 bytes encode a valid curve point or match a spent output. |
| `validate_key_path_signature` | Signature is 64 bytes, or 65 bytes with a non-zero explicit sighash byte. | Key-path witness element shape. | Schnorr encoding, signature validity, sighash semantics, key tweaking, or transaction context. |
| `inspect_taproot_witness` | Empty-witness rejection, annex position detection, key/script path positions, and control-block shape. | Structural witness classification. | Annex policy, commitment verification, script parsing, or execution. |
| `validate_taproot_witness` | The same shape checks plus current-leaf-version support. | Fail-closed current structural contract. | Taproot cryptography and BIP-342 runtime validity. |
| `inspect_control_block` | `33 + 32m` length, `m <= 128`, parity bit, masked leaf version, opaque internal-key position. | BIP-341 control-block shape. | Curve lifting, internal-key validity, Merkle hashes, tweak/commitment matching, and parity matching. |
| `validate_control_block` | The inspect checks plus current BIP-342 leaf-version support. | Current structural handoff. | All cryptographic and script-path checks. |
| `validate_miniscript_handoff` | Version, supported context, public metadata relationships, and capability ownership. | Static metadata/handoff contract. | Miniscript parsing, compilation, satisfaction, execution, and cryptographic verification. |

## Standards and ownership matrix

The status values are deliberately separate:

- **Supported:** Core can validate the stated public structural invariant.
- **Unsupported:** the value is well-formed but outside this versioned Core API.
- **Downstream-owned:** Core can identify the boundary, but a transaction parser, compiler,
  cryptographic verifier, or runtime must decide the result.

| Surface | Standard requirement | Core status | Downstream responsibility |
| --- | --- | --- | --- |
| P2TR output shape | BIP-341 witness version `1` with a 32-byte witness program. | **Supported** shape check. | Validate the containing output and use the program as the commitment target. |
| Key-path witness | After annex removal, exactly one witness element is the signature. | **Supported** position/length check. | Verify BIP-340 signature against the output key and select the correct BIP-341 sighash. |
| Key-path signature length | 64-byte Schnorr signature, or 65 bytes with a non-zero sighash byte. | **Supported** length and zero-byte rejection. | Interpret the sighash byte and verify the signature in transaction context. |
| Annex | If at least two witness elements remain and the last begins with `0x50`, it is the annex. | **Supported** position classification only. | Apply annex policy and include the annex in the correct sighash calculation. |
| Script-path positions | The penultimate post-annex element is the script; the last is the control block. | **Supported** position classification. | Parse the script, apply leaf-version rules, verify the control block, and execute the script. |
| Control-block shape | Length is `33 + 32m`, with `m` in `0..=128`; maximum is `4129` bytes. | **Supported** structural bound. | Lift the internal key, calculate the TapLeaf/Merkle/TapTweak commitment, compare to the output key, and check parity. |
| Control-block internal key | The 32-byte internal-key encoding is interpreted as a BIP-340 x-only key by BIP-341. | **Opaque bytes only.** | Validate the curve point and all commitment relationships. |
| Current Tapscript leaf | The masked leaf version is `0xc0`; encoded control bytes `0xc0` and `0xc1` differ by parity. | **Supported** classification. | Apply all BIP-342 execution rules. |
| Future/unknown leaf | Future leaf versions are upgrade hooks and are not universally malformed by BIP-341. | **Downstream-owned**: structurally inspectable, fail-closed validation error. | Implement the future leaf semantics or an explicit policy for accepting/rejecting them. |
| `OP_SUCCESSx` | BIP-342 `OP_SUCCESSx` can make a script valid before normal parsing/execution. | **Downstream-owned.** | Parse the Tapscript in context and enforce the consensus rule. |
| `MINIMALIF` | Tapscript makes `OP_IF`/`OP_NOTIF` branch arguments consensus-constrained. | **Downstream-owned.** | Execute or interpret the script with BIP-342 rules. |
| `CHECKSIGADD` | BIP-342 adds opcode `0xba` for threshold-style Tapscript policies. | **Metadata context check only.** | Parse and execute the opcode, enforce signature semantics, and account for sigops budget. |
| Tapscript resources | BIP-342 changes script-size, non-push-opcode, sigops, stack-count, and stack-element rules. | **Downstream-owned.** | Enforce limits against the complete transaction and runtime state. |
| Miniscript metadata | Public policy kind, threshold/signers, satisfaction bound, and feature flags can be handed off. | **Supported** static relationship checks. | Derive metadata from a real policy and preserve the source policy/descriptor separately. |
| Miniscript compilation | Policy-to-Miniscript/Script compilation and descriptor expansion. | **Unsupported in Core.** | SDK/wallet/compiler owner. |
| Miniscript satisfaction | Witness construction, preimages, signatures, and timelock satisfaction. | **Downstream-owned.** | SDK/wallet and transaction-aware policy owner. |
| Miniscript execution | Script interpreter and satisfaction validity. | **Downstream-owned.** | BIP-342/Tapscript runtime owner. |

### BIP-341 control blocks versus BIP-110 size policy

These are different contracts and must not be conflated:

- **BIP-341 shape:** `33 + 32m` bytes, `m` in `0..=128`, maximum `4129` bytes. This is validated
  by `inspect_control_block`/`validate_control_block`.
- **BIP-110 proposal size policy:** the existing preflight contract has a separate explicitly
  classified control-block measurement with a `257`-byte proposal limit. That is a policy-size
  check, not a BIP-341 shape check. A control block can satisfy one boundary and fail the other.

The BIP-110 distinction is documented in [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md).
The Taproot module does not reuse, widen, or reinterpret the BIP-110 `257`-byte field.

## Deterministic vector matrix

The integration vectors are in
[`tests/bip341_bip342_miniscript.rs`](../../tests/bip341_bip342_miniscript.rs).

| Vector | Expected result |
| --- | --- |
| v1 program with 32 bytes | Supported structural result with `structural_only` claim. |
| v1 program with 31 bytes | `malformed / witness_program_wrong_length`. |
| v0 program with 32 bytes | `unsupported / unsupported_witness_version`. |
| 64-byte key-path signature | Supported shape. |
| 65-byte key-path signature with `0x01` sighash byte | Supported shape with explicit byte recorded as metadata. |
| 65-byte key-path signature with `0x00` sighash byte | `malformed / key_path_signature_zero_sighash`. |
| 33-byte `0xc0` control block | Current Tapscript, even parity, zero Merkle nodes. |
| 33-byte `0xc1` control block | Current Tapscript, odd parity, zero Merkle nodes. |
| `33 + 32*128` control block | Supported maximum BIP-341 structural depth. |
| 32-byte, 34-byte, and depth-129 control blocks | Typed malformed errors. |
| `0xe0`/`0xe1` control byte | Structurally inspectable future leaf; fail-closed validation is downstream-owned. |
| Key-path witness with annex | Annex is classified at the final witness position. |
| Script-path witness with arguments, leaf, control block, and annex | Positions and stack-argument count are deterministic. |
| Empty witness | `malformed / empty_witness`. |
| `thresh(2, ...)`-style public metadata | Supported static handoff when context/capabilities are valid. |
| Timelock-only metadata | Supported when timelock flag and zero signer counts agree. |
| Required signatures greater than candidate signers, or max satisfaction elements less than required signatures | `malformed / invalid_miniscript_metadata`. |
| `CHECKSIGADD` metadata in SegWit v0 context | `malformed / miniscript_context_mismatch`. |
| Compilation, satisfaction, execution, or cryptographic-verification request | `downstream_owned / downstream_owned_miniscript_capability`. |

No vector uses a real secret, signature verification, curve operation, transaction, or interpreter.

## Wire stability and error behavior

Shared structs derive `Serialize`/`Deserialize` with explicit snake-case enum values. The tests
pin representative JSON shapes for:

- `BitcoinBoundaryError` category/code pairs;
- the `structural_only` validation claim; and
- public Miniscript metadata round trips.

The error model is intentionally small and stable:

| Category | Meaning | Example |
| --- | --- | --- |
| `malformed` | The supplied bytes/metadata violate a structural relationship. | `control_block_length_misaligned` |
| `unsupported` | The value is valid in a broader protocol but not supported by this API version. | `unsupported_witness_version` |
| `downstream_owned` | Core identifies a future or runtime/crypto boundary and refuses to imply a result. | `unknown_taproot_leaf_version` |

Callers must branch on the category/code and must not convert `downstream_owned` into a successful
verification result. Error formatting contains no raw input data.

## SDK handoff

The enclave SDK remains the owner of concrete key-path tweak/signing and hardware-backed policy
flows. The current downstream evidence is `conxius-enclave-sdk/src/protocol/bitcoin.rs`, where
`TaprootManager` derives Taproot output keys and signs supplied Taproot-related hashes. The Core
contract can be used to carry public structural metadata before or alongside that handoff. Full
script-path execution and Miniscript compilation are not implied by this module and remain explicit
SDK/downstream work.

Recommended downstream sequence:

1. Parse the transaction and classify the witness/output context in the wallet or transaction
   adapter.
2. Call `validate_p2tr_witness_program` and `validate_taproot_witness` before treating the shape as
   eligible for a current structural handoff.
3. Treat the returned claim as structural only; perform BIP-341 cryptographic checks in the
   transaction-aware verifier.
4. Use `validate_miniscript_handoff` only for compiler-produced public metadata. Keep policy text,
   descriptors, keys, preimages, signatures, and satisfaction witnesses in the owning downstream
   boundary.
5. Apply BIP-342 execution/resource rules and any BIP-110 policy checks in the same downstream
   transaction/runtime context; do not substitute one for the other.

## Unresolved gaps

- No Core transaction or witness parser exists; callers must supply correctly classified elements.
- No BIP-341 TapLeaf/Merkle/TapTweak hashing or output-key commitment validation exists here.
- No Schnorr signature, key, sighash, or parity cryptographic verification exists here.
- No BIP-342 interpreter, `OP_SUCCESSx` handling, `MINIMALIF`, `CHECKSIGADD`, or resource-budget
  enforcement exists here.
- No Miniscript parser, compiler, descriptor expander, satisfier, or executor exists here.
- Future leaf versions are intentionally observable but rejected by current validation until a
  downstream owner defines their semantics.
- This document records the Core/SDK boundary; it does not claim that every downstream consumer
  currently enforces each handoff.

## References

- [BIP-341: Taproot](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki)
- [BIP-342: Validation of Taproot Scripts](https://github.com/bitcoin/bips/blob/master/bip-0342.mediawiki)
- [BIP-379: Miniscript](https://github.com/bitcoin/bips/blob/master/bip-0379.md)
- [BIP-340: Schnorr signatures](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki)
- [Core BIP-110 alignment](../BIP110_ALIGNMENT.md)
- [Core Bitcoin signing handoff](../signing/bitcoin.md)
- [Architecture boundaries](../ARCHITECTURE_BOUNDARIES.md)
