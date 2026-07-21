# BIP-110 Compliance Matrix

> **Repository scope:** Core size-contract documentation only. This document does not claim that
> `lib-conxian-core` implements a complete Bitcoin consensus validator, transaction parser, script
> interpreter, Taproot verifier, or downstream enforcement layer.
>
> **Proposal status:** The canonical BIP-110 text is marked `Complete`. Under [BIP-3's status
> definitions](https://github.com/bitcoin/bips/blob/master/bip-0003.md#progression-through-bip-statuses),
> `Complete` is distinct from `Deployed`: it records a mature proposal recommended for adoption,
> implementation, or deployment, not proof that the proposed consensus rules are active on a
> network. This repository does not infer activation, activation height, signaling, or expiry.
>
> **Last updated:** 2026-07-21

## Executive summary

The current Core contract represents the neutral, size-bearing subset of BIP-110:

- applicable pushdata payloads are limited to **256 bytes**;
- complete OP_RETURN output ScriptPubKeys are limited to **83 bytes**;
- complete non-OP_RETURN output ScriptPubKeys are limited to **34 bytes**; and
- applicable script-argument witness items are limited to **256 bytes**; and
- explicitly classified Taproot control blocks are limited to **257 bytes**.

CORE-005 adds a versioned, phase-aware preflight envelope around those measurements. The envelope
is a platform-neutral contract for downstream adapters; it does not make Core a transaction parser
or claim that any downstream consumer already enforces the contract.

`Bip110Compliance`, `Bip110Limits`, and `Bip110TransactionShape` validate supplied byte-size
metadata. A downstream adapter must parse the transaction, identify the script context, apply the
exceptions below, and populate the shape. The adapter, SDK, Wallet, Gateway, or Nexus then owns
the decision to reject a transaction before signing, broadcasting, routing, or observation.

The Core contract therefore answers **“does this supplied size metadata meet the configured
limits?”** It does not answer **“is this transaction valid under every BIP-110 rule?”**

## Canonical source and terminology

The matrix follows the current canonical texts, not an inferred activation policy:

| Source | Relevant semantics |
| --- | --- |
| [BIP-110 Specification](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki#specification) | Seven proposed temporary consensus rules. |
| [BIP-110 UTXO grandfathering](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki#utxo-grandfathering) | Inputs spending pre-activation UTXOs are exempt during the deployment; rules cease after expiry. |
| [BIP-110 specification nuance](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki#specification-nuance) | Defines script-argument witness items and the script/control-block/annex exceptions. |
| [BIP-110 deployment](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki#deployment) | Proposal parameters and state transitions; not an activation observation consumed by Core. |
| [BIP-3](https://github.com/bitcoin/bips/blob/master/bip-0003.md) | Separates the `Complete` proposal status from `Deployed` evidence of active use or activation. |
| [BIP-16](https://github.com/bitcoin/bips/blob/master/bip-0016.mediawiki#specification) | Defines the redeemScript push and its separate P2SH validation rules. |
| [BIP-141](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki#witness-program) | Defines witness programs, v0 witness scripts, witness stacks, and undefined versions. |
| [BIP-341](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#script-validation-rules) | Defines Taproot annex detection, script-path stacks, leaf versions, and control-block lengths. |
| [BIP-342](https://github.com/bitcoin/bips/blob/master/bip-0342.mediawiki#specification) | Defines Tapscript execution, OP_SUCCESSx, and the execution context for OP_IF/OP_NOTIF. |

## Current Core contract

| Core field or API | Measurement and limit | What Core does not infer |
| --- | --- | --- |
| `Bip110Limits::max_pushdata_bytes` / `pushdata_sizes_bytes` | Payload bytes for each applicable pushdata element; `<= 256` passes. | Which opcode was parsed, whether a push is in a script or scriptSig, and whether a BIP-110 exception applies. |
| `Bip110Limits::max_op_return_bytes` / `op_return_script_pubkey_sizes_bytes` | Full serialized output ScriptPubKey bytes for each classified OP_RETURN output; `<= 83` passes. | Whether the output is new, whether its first opcode is OP_RETURN, and whether the output is covered by activation or grandfathering. |
| `Bip110Limits::max_script_pubkey_bytes` / `non_op_return_script_pubkey_sizes_bytes` | Full serialized ScriptPubKey bytes for each classified non-OP_RETURN output; `<= 34` passes. | Output classification, transaction validity, and deployment state. |
| `Bip110Limits::max_witness_element_bytes` / `witness_element_sizes_bytes` | Bytes in each applicable script-argument witness item; `<= 256` passes. | Witness version, key-path versus script-path selection, annex/control-block identification, and script execution. |
| `Bip110Compliance::new()` / `Bip110Compliance::default()` / `Bip110Compliance::disabled()` | `new()` enables the canonical size contract; `default()` and `disabled()` intentionally disable it. | Network consensus activation. Enabling this Rust validator is a caller choice, not a claim about Bitcoin network state. |
| `Bip110PreflightMeasurements::taproot_control_block_sizes_bytes` | Complete serialized control-block witness-item sizes, kept separate from script-argument witness items; `<= 257` passes. | BIP-341 control-block shape, witness position, Merkle commitment, internal-key parity, and cryptographic validity. |
| `Bip110PreflightRequest` / `Bip110PreflightResult` | Versioned request/result envelope with explicit phase/source provenance, context, fixed-width `u64` measurements, and indexed findings. | Transaction construction, serialization, parsing, script execution, deployment state, and downstream rejection policy. |

The validators use an inclusive boundary: `size <= max` is compliant and `size > max` produces
the corresponding structured violation. Vector validation preserves the current deterministic
ordering: pushdata, OP_RETURN ScriptPubKeys, non-OP_RETURN ScriptPubKeys, witness elements, then
Taproot control blocks.

The executable sources are [`src/control_model/bip110.rs`](../src/control_model/bip110.rs) for the
legacy limits and [`src/control_model/bip110_preflight.rs`](../src/control_model/bip110_preflight.rs)
for the versioned preflight contract and control-block boundary.

## CORE-005 preflight contract

`BIP110_PREFLIGHT_API_VERSION` is a fixed-width `u16` with the initial value `1`. A
`Bip110PreflightRequest` contains:

- `phase`: `pre_construction` for intended serialized surfaces before final bytes exist, or
  `post_serialization` for measurements taken from the finalized serialized transaction;
- `measurements.source`: `caller_classified` for pre-construction measurements or
  `serialized_transaction` for post-serialization measurements. The validator returns a stable
  `phase_mismatch` error when these labels disagree;
- `context`: a stable string-valued operation context; only `bitcoin_transaction` is supported in
  this contract version; and
- `measurements`: four occurrence-ordered ordinary `u64` vectors plus the separate
  `taproot_control_block_sizes_bytes` vector. An omitted measurement object is distinct from
  present empty vectors and returns `missing_measurement_data`.

Pre-construction is caller-classified planning metadata; it is not full serialized transaction
validation. Post-serialization measurements must be classified from finalized bytes, but Core still
does not parse, serialize, or cryptographically validate those bytes.

The byte units are authoritative in both phases:

| Request field | Authoritative measurement |
| --- | --- |
| `pushdata_sizes_bytes` | Payload bytes carried by each applicable pushdata operation, excluding its opcode and length prefix. |
| `op_return_script_pubkey_sizes_bytes` | The complete serialized OP_RETURN output ScriptPubKey, including `OP_RETURN`, push opcodes, and push prefixes. |
| `non_op_return_script_pubkey_sizes_bytes` | The complete serialized non-OP_RETURN output ScriptPubKey, including all script bytes and push prefixes. |
| `witness_element_sizes_bytes` | Each applicable script-argument witness element, excluding item-length prefixes, other witness items, and total witness serialization. |
| `taproot_control_block_sizes_bytes` | Each complete serialized Taproot control-block witness item, excluding the item-length prefix and kept out of the ordinary witness vector. |

The preflight implementation checks every `u64` measurement before converting it to the existing
`usize`-based `Bip110TransactionShape`; values are never truncated. It then composes with an
enabled `Bip110Compliance` instance, so canonical limits remain defined in one place. A disabled
`Bip110Compliance` (including the intentionally disabled `Default` value) is rejected when used to
construct a preflight validator. The ordinary `Bip110TransactionShape` and
`Bip110Compliance` APIs remain unchanged; control-block size checking is additive to the preflight
wrapper and does not claim that a control block is structurally or cryptographically valid.

### Context support matrix

| Context wire value | Status in API version 1 | Required behavior |
| --- | --- | --- |
| `bitcoin_transaction` | **Supported** | The caller asserts that all ordinary vectors are fully classified; a separately classified control-block vector may also be supplied. Empty vectors are valid only when the generic transaction has zero constrained occurrences and the measurement object is present. |
| `taproot`, `tapleaf`, `tapscript`, `taproot_script_path`, `taproot_key_path` | **Known but unsupported** | Return `unsupported_context`; do not infer annex, control-block, key-path, or script-path semantics from generic vectors. |
| `miniscript`, `dlc` and its funding/refund/CET roles, `lightning` and its transaction roles, `rgb`/`rgb_anchor`, `babylon` and its transaction roles, `fedimint`, `stacks`/sBTC transaction roles, `liquid` | **Known but unsupported** | Return `unsupported_context` until the owning context contract is defined. |
| Any other string | **Unknown** | Preserve the string through serde round trips and return `unknown_context`. Empty vectors do not make an unknown context compliant. |

`Bip110PreflightResult.findings` is deterministic: structural/request errors are emitted first,
followed by size findings in canonical category order—pushdata, OP_RETURN ScriptPubKey,
non-OP_RETURN ScriptPubKey, witness, then Taproot control block—and occurrence order within each vector. Each size finding
contains a stable code, field, zero-based `u64` index, actual bytes, and maximum bytes. The result
is compliant only when the findings vector is empty; unsupported API versions, unknown or
unsupported contexts, missing measurement data, phase/source mismatches, conversion failures, and
size violations all fail closed.

### Downstream consumption notes

The following are integration handoffs, not claims that those consumers currently enforce this
contract:

1. **SDK #194 (`conxius-enclave-sdk`)** owns the remaining Core/SDK type and adapter alignment;
   its downstream signing work should use the pre-construction result as a rejection gate and
   preserve post-serialization findings for final-byte verification. The separate SDK #179
   BIP-322/BIP-110 signing work remains outside this Core change.
2. **Gateway #245 (`conxian-gateway`)** should carry the version, phase, context, and ordered
   findings through orchestration without converting them into warnings-only status.
3. **Wallet #381 (`conxius-wallet`)** should populate the fully-classified generic Bitcoin context
   and reject any non-compliant or unsupported result before broadcast.

Core defines these request/result types and their fail-closed semantics only; it does not claim
that SDK #194 (or its related SDK #179 signing work), Gateway #245, or Wallet #381 have implemented
enforcement.

## Proposed-rule matrix

The status column uses three intentionally separate classifications:

- **Core size contract:** the current public Core fields and validator can check the stated byte
  limit after an adapter supplies correctly classified metadata.
- **Adapter/parser-owned:** the rule can be evaluated only after a downstream component parses the
  transaction and identifies the relevant Bitcoin context; no Core parser is implied.
- **Unsupported/not represented:** the current Core API has no field or execution model for the
  rule. A future adapter or preflight layer may enforce it without changing this matrix's scope.

| BIP-110 proposal rule | Current classification | Exact boundary and remaining ownership |
| --- | --- | --- |
| **1. New output ScriptPubKeys:** more than 34 bytes is invalid unless the first opcode is OP_RETURN, where up to 83 bytes is valid. | **Core size contract** for the two size fields; **adapter/parser-owned** for output age, opcode classification, and deployment context. | Measure the complete output ScriptPubKey, including opcodes and push prefixes. Core checks 83 for classified OP_RETURN outputs and 34 for classified non-OP_RETURN outputs. It does not decide what “new” means or apply UTXO grandfathering. |
| **2. OP_PUSHDATA* payloads and script-argument witness items:** more than 256 bytes is invalid, except for the BIP-16 redeemScript push. | **Core size contract** for supplied `pushdata_sizes_bytes` and applicable `witness_element_sizes_bytes`; **adapter/parser-owned** for applicability and exceptions. | Measure payload/item bytes, not the encoding prefix. Exclude the redeemScript push, witness scripts, Tapleaf scripts, control blocks, annexes, Taproot key-path signatures, and undefined-version witness stacks as described below; then classify any remaining script-argument items and applicable inner pushdata occurrences. |
| **3. Spending undefined witness or Tapleaf versions:** spending versions other than the defined BIP-141, BIP-341, or P2A cases is invalid; creating such outputs remains valid. | **Unsupported/not represented; adapter/parser-owned.** | The Core shape has no witness-version, Tapleaf-version, output-creation, or spend-context field. A downstream validator must distinguish creation from spending and must not turn this rule into a generic 256-byte witness check. |
| **4. Taproot annex:** any witness stack with an annex is invalid. | **Unsupported/not represented; adapter/parser-owned.** | BIP-341 identifies an annex from the last witness element when at least two elements remain and its first byte is `0x50`. The Core shape has no annex field and cannot identify or reject it. |
| **5. Taproot control block:** a control block larger than 257 bytes is invalid. | **Core preflight size contract** for an explicitly classified control-block measurement; **adapter/parser-owned** for witness position, BIP-341 shape, commitment, and cryptographic validation. | `taproot_control_block_sizes_bytes` is separate from `witness_element_sizes_bytes`; `257` passes and `258` produces `taproot_control_block_exceeds_limit`. Size-admissible does not mean structurally or cryptographically valid. |
| **6. Tapscript OP_SUCCESSx:** any OP_SUCCESSx opcode anywhere, even unexecuted, is invalid. | **Unsupported/not represented; adapter/parser-owned.** | Requires decoding a BIP-342 Tapscript in the correct Taproot script-path context. Core has no opcode stream or Tapscript interpreter. |
| **7. Tapscript OP_IF/OP_NOTIF:** executing either opcode is invalid regardless of result. | **Unsupported/not represented; adapter/parser-owned.** | Requires identifying the BIP-342 Tapscript execution path and observing executed opcodes. A static size check cannot establish this rule. |
| **UTXO grandfathering:** inputs spending UTXOs created before activation are exempt during the deployment; after expiry, UTXOs are unrestricted again. | **Unsupported/not represented; deployment/parser-owned.** | Enforcement requires the deployment state, activation height, spent-output creation height, and expiry state. No height, signaling, or expiry field exists in Core. |

The BIP-110 GBT deployment name and signaling requirements are deployment mechanics rather than
size metadata. They are likewise **unsupported/not represented** here; the [BIP-110 deployment
section](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki#deployment) is not an
activation claim.

## Byte units and context exceptions

All measurements in the Core contract are byte counts. They are not character counts, fee units,
weight units, transaction totals, or an aggregate witness serialization length.

| Surface | Count this | Do not count this | Ownership note |
| --- | --- | --- | --- |
| Pushdata payload | The bytes carried by one applicable pushdata operation. | The opcode and its CompactSize/direct-push length prefix. | The adapter parses the script and decides whether the occurrence is subject to rule 2. |
| OP_RETURN output | The complete serialized output ScriptPubKey, including `OP_RETURN`, push opcodes, and push prefixes. | Do not substitute only the OP_RETURN payload length. | Core checks the supplied full length against 83. |
| Non-OP_RETURN output | The complete serialized output ScriptPubKey, including all script bytes and push prefixes. | Do not measure only a hash, key, or logical policy component. | Core checks the supplied full length against 34. |
| Script-argument witness item | The bytes of one witness stack element that is placed on the script interpreter's initial stack. | The item-length prefix, the other witness elements, or the total witness serialization. | Core checks only items the adapter classifies as applicable rule-2 inputs. |
| Taproot control block | The complete control-block witness item, whose BIP-341 shape is `33 + 32m` bytes and whose BIP-110 size cap is 257 bytes. | Do not fold it into the 256-byte script-argument item vector. | Core preflight checks only the supplied size; adapters own position, shape, commitment, and cryptographic validation. |

### Exceptions and related script surfaces

- **BIP-16 redeemScript push:** BIP-110 explicitly exempts the redeemScript push in a BIP-16
  `scriptSig` from its 256-byte rule-2 pushdata limit. That does not erase BIP-16's separate
  serialized redeemScript constraints, including BIP-16's 520-byte consequence for the serialized
  redeemScript, or make the script body opaque to other applicable rules. The adapter must not
  place the exempt outer redeemScript push in `pushdata_sizes_bytes`; it remains responsible for
  BIP-16 parsing and any applicable inner pushdata checks.
- **Witness scripts:** Under BIP-141, a v0 P2WSH witness script is popped from the witness before
  the remaining stack is executed. The script itself is not a script-argument witness item. Do
  not place the whole witness script in `witness_element_sizes_bytes`; parse its inner pushdata
  operations when applying rule 2 and retain separate BIP-141 script semantics outside this Core
  size contract.
- **Tapleaf scripts:** Under BIP-341, the Tapleaf/Tapscript is the penultimate script-path witness
  element after annex handling. It is script, not a script-argument witness item. Its serialized
  script bytes are not a 256-byte witness-item measurement, while applicable pushdata operations
  within the script remain subject to the rule-2 interpretation. BIP-342 execution rules remain
  adapter/parser-owned.
- **Miniscript expansions:** Core does not compile Miniscript or measure policy complexity. Once a
  downstream compiler produces a Bitcoin script, treat the resulting Tapleaf as script bytes,
  measure applicable pushdata operations inside it, measure actual satisfaction arguments as
  script-argument witness items, and keep control blocks separate. Leaf count, tree depth,
  semantic validity, and compiler choices remain part of the BIP-341/BIP-342 and Miniscript
  handoff in [issue #178](https://github.com/Conxian/lib-conxian-core/issues/178).
- **Taproot control blocks:** The control block is the final script-path witness element after
  annex handling. Under BIP-110, its maximum proposed size is 257 bytes, not 256: `33 + 32*7`
  consists of one control byte, a 32-byte internal key, and seven 32-byte Merkle-path sibling
  hashes. For a balanced tree, that depth-7 path corresponds to at most `2^7 = 128` leaves.
  BIP-341's generic `33 + 32m` form allows `0 <= m <= 128` (up to 4,129 bytes); BIP-110
  supplies the stricter 257-byte cap. It is not a script argument and is represented only by the
  separate fixed-width preflight control-block vector.
- **Taproot annexes:** BIP-341 treats a last witness element beginning with `0x50` as an annex
  when the witness has at least two elements. It is removed before script-path inputs are passed
  to the interpreter. BIP-110 would reject the spend rather than apply the 256-byte item limit;
  Core has no annex discriminator.
- **Taproot key-path signatures:** BIP-110's rationale excludes key-path signatures from
  “script argument witness items”; BIP-341's signature validation accepts exactly 64 or 65 bytes.
  The Core size contract does not model key-path selection or signature validation.
- **Undefined witness/Tapleaf versions:** BIP-141 permits outputs for future witness versions
  without interpreting their witness stacks. BIP-110 proposes rejecting spends of undefined
  witness/Tapleaf versions while leaving output creation valid. Core has no version or deployment
  state and must not treat every unknown witness item as a normal 256-byte argument.
- **Grandfathering and expiry:** The proposed exception is attached to the spent UTXO's creation
  height and the temporary deployment state, not to the byte shape itself. Core does not expose
  an activation-height or expiry API, so this logic belongs in a deployment-aware adapter.

## Boundary vectors

For the four ordinary fields and the separate control-block field, the validator's predicate is
inclusive: the exact limit passes
and the next byte fails. A size of `0` also satisfies the size predicate, although a zero-length
ScriptPubKey or witness item may be invalid for other Bitcoin reasons that this contract does not
parse. Empty vectors mean that no occurrence of that classified surface was supplied; the legacy
optional OP_RETURN argument maps to an empty OP_RETURN vector when absent.

| Surface | Exact-limit vector | Limit+1 vector | Current Core execution status |
| --- | --- | --- | --- |
| Applicable pushdata payload | `256` → compliant | `257` → `PushdataExceedsLimit` | **Executable now** through `validate_pushdata` or `pushdata_sizes_bytes`. |
| Complete OP_RETURN output ScriptPubKey | `83` → compliant | `84` → `OpReturnExceedsLimit` | **Executable now** through `validate_op_return` or `op_return_script_pubkey_sizes_bytes`. These are full ScriptPubKey sizes, not payload sizes. |
| Complete non-OP_RETURN output ScriptPubKey | `34` → compliant | `35` → `ScriptPubKeyExceedsLimit` | **Executable now** through `validate_script_pubkey` or `non_op_return_script_pubkey_sizes_bytes`. |
| Applicable script-argument witness item | `256` → compliant | `257` → `WitnessElementExceedsLimit` | **Executable now** through `validate_witness_element` or `witness_element_sizes_bytes`, after adapter classification. |
| Taproot control block | `257` → size-admissible under BIP-110 and exact BIP-341 length form `33 + 32*7` | `258` → `TaprootControlBlockExceedsLimit` | **Executable now** through `taproot_control_block_sizes_bytes`; size alone cannot prove BIP-341 shape or cryptographic validity. |

The existing Rust tests also exercise multiple simultaneous ordinary/control-block violations,
disabled compliance, JSON round trips, phase/source mismatches, missing measurements, specialized
context rejection, and deterministic vector ordering.

## Protocol and chain-family mapping

BIP-110 applies to Bitcoin transaction validation surfaces. Protocol bytes remain outside the
matrix unless they are actually serialized into a Bitcoin output ScriptPubKey, a Bitcoin script,
or an applicable Bitcoin witness item.

| Protocol | Measure on the Bitcoin surface | Do not classify as Bitcoin pushdata |
| --- | --- | --- |
| **Lightning** | Commitment, closing, justice, HTLC-timeout, and penalty transactions when they are serialized for Bitcoin; measure their actual outputs, scripts, and applicable witness arguments. | Channel state, invoices, onion packets, gossip, and off-chain commitment data. |
| **RGB** | Bitcoin anchor/commitment outputs and any RGB-related Bitcoin script or witness bytes actually published to L1. | Client-side validation, seals, state transitions, consignments, and off-chain RGB data. |
| **Babylon** | Bitcoin staking, delegation, unbonding, withdrawal, or checkpoint transactions and their actual Bitcoin script/witness surfaces. | BTC header observations, EOTS proofs, Babylon messages, and off-chain staking/finality state. |
| **Fedimint** | Bitcoin peg-in, peg-out, and federation-controlled Bitcoin transactions; measure only the serialized Bitcoin surfaces. | Guardian consensus, federation state, e-cash notes, and federation protocol messages. |
| **Stacks / sBTC** | Bitcoin-side peg-in, peg-out, mint, burn, or withdrawal transactions and their actual Bitcoin outputs/scripts/witnesses. | Stacks transactions, Clarity execution, sBTC ledger state, and Stacks-side consensus data. |
| **Liquid** | Bitcoin-side peg-in/peg-out or federation transaction surfaces that are actually serialized on Bitcoin L1. | Liquid/Elements sidechain transactions, confidential-asset proofs, and federation state not present in a Bitcoin transaction. |
| **DLC** | Funding, refund, and CET transactions when serialized on Bitcoin, including their actual scripts and witness arguments. | Oracle attestations, outcome data, contract metadata, and off-chain coordination. |

The adapter must not concatenate or re-label off-chain, sidechain, federation, or protocol-state
bytes as Bitcoin pushdata merely because the protocol ultimately settles or anchors on Bitcoin.

## Downstream handoff and non-goals

1. A transaction-aware adapter measures raw Bitcoin bytes, applies the BIP-16/BIP-141/BIP-341/BIP-342
   context rules above, excludes non-applicable items, and fills every relevant vector in
   `Bip110TransactionShape`.
2. The adapter or a future neutral preflight layer reports unsupported context instead of silently
   treating an unclassified Taproot, Miniscript, DLC, or future witness surface as compliant. See
   [issue #176](https://github.com/Conxian/lib-conxian-core/issues/176) for that preflight contract.
3. SDK and Wallet signing flows own concrete transaction construction and fail-closed enforcement;
   Gateway and Nexus own orchestration and observation. See [issue #175](https://github.com/Conxian/lib-conxian-core/issues/175).
4. Taproot, Tapscript, and Miniscript invariants remain a separate audit and handoff. See [issue
   #178](https://github.com/Conxian/lib-conxian-core/issues/178).
5. The neutral structural contract for that handoff is documented in
   [`docs/bitcoin/TAPROOT_MINISCRIPT_SUPPORT.md`](bitcoin/TAPROOT_MINISCRIPT_SUPPORT.md). Its
   BIP-341 control-block bound (`33 + 32m`, `m <= 128`) is distinct from this document's separate
   BIP-110 `257`-byte policy measurement.

This matrix does **not** implement transaction parsing, BIP-341 cryptography, BIP-342 execution,
Miniscript compilation, BIP-322 verification, transaction builders, signing, fee routing, network
I/O, activation-height logic, UTXO persistence, or downstream policy.

## History and related work

- [Issue #168](https://github.com/Conxian/lib-conxian-core/issues/168) and [PR #169](https://github.com/Conxian/lib-conxian-core/pull/169) introduced the original Core BIP-110 constants and validator.
- [PR #184](https://github.com/Conxian/lib-conxian-core/pull/184) added the current serializable limits and transaction-shape contract.
- [PR #189](https://github.com/Conxian/lib-conxian-core/pull/189) hardened the merged contract coverage.
- [PR #194](https://github.com/Conxian/lib-conxian-core/pull/194) added the evidence-backed compliance matrix and clarified proposal/deployment status.
- [PR #201](https://github.com/Conxian/lib-conxian-core/pull/201) added the versioned, fail-closed preflight contract.
- [SDK issue #194](https://github.com/Conxian/conxius-enclave-sdk/issues/194) records the remaining SDK/Core alignment and adapter work; it is downstream of this protocol contract.
- [Parent issue #173](https://github.com/Conxian/lib-conxian-core/issues/173) tracks the broader research umbrella.
- [Issue #179](https://github.com/Conxian/lib-conxian-core/issues/179) tracks this compliance-matrix follow-up. This completion change supplies the remaining legacy optional-OP_RETURN regression coverage; issue status is tracked independently.

## References

- [BIP-110: Reduced Data Temporary Softfork](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki)
- [BIP-3: Updated BIP Process](https://github.com/bitcoin/bips/blob/master/bip-0003.md)
- [BIP-16: Pay to Script Hash](https://github.com/bitcoin/bips/blob/master/bip-0016.mediawiki)
- [BIP-141: Segregated Witness](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki)
- [BIP-341: Taproot](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki)
- [BIP-342: Validation of Taproot Scripts](https://github.com/bitcoin/bips/blob/master/bip-0342.mediawiki)
- [Core API reference](API.md)
- [Core trust/control model](../src/control_model/)
