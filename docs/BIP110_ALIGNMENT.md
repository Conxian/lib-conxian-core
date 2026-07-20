# BIP-110 Alignment for Conxian Ecosystem

> **Status**: Active | **Last Updated**: 2026-07-20 | **Version**: 1.0

## Executive Summary

This document records the explicit size-policy subset represented by
`lib-conxian-core`. It is not a claim that this crate implements a complete
BIP-110 consensus model, raw-transaction parser, or script verifier.

## What is BIP-110?

BIP-110, titled "Reduced Data Temporary Softfork" by Dathon Ohm, is a proposal
that seeks to:

1. **Temporarily limit data fields at consensus level** to reject data storage as a supported use case
2. **Refocus Bitcoin on monetary use** by setting it "back on the path to becoming the world's money"
3. **Restore smaller OP_RETURN policy limits** at consensus (83-byte limit)

### Technical Rules

The following table describes the size-policy inputs that this repository can
validate after a downstream adapter has classified the transaction. It does
not define all BIP-110 script-context rules.

| Core policy input | Limit | Shape interpretation |
|-------------------|-------|----------------------|
| OP_RETURN output ScriptPubKey | 83 bytes | Full output ScriptPubKey bytes, not only the payload |
| Non-OP_RETURN ScriptPubKey | 34 bytes | Full ScriptPubKey bytes for each classified output; not a claim about any particular output type's serialized size |
| Pushdata element | 256 bytes | Every supplied pushdata element subject to this limit |
| Witness element | 256 bytes | Every supplied witness element subject to this limit |
| Tapleaf formats | Context-sensitive | Not represented by the size-only shape; adapters handle applicable rules |

## Canonical Core Contract

`lib-conxian-core` owns the deterministic size-policy limits and transaction
shape contract in `src/control_model/bip110.rs`. The public types are
re-exported from `lib_conxian_core::control_model`:

| Type/field | Meaning |
|------------|---------|
| `Bip110Limits::max_pushdata_bytes` | Maximum size of one pushdata element; canonical default is 256 bytes |
| `Bip110Limits::max_op_return_bytes` | Maximum full output ScriptPubKey size for each OP_RETURN entry; canonical default is 83 bytes |
| `Bip110Limits::max_script_pubkey_bytes` | Maximum full ScriptPubKey size for each non-OP_RETURN entry; canonical default is 34 bytes |
| `Bip110Limits::max_witness_element_bytes` | Maximum size of one witness element; canonical default is 256 bytes |
| `Bip110TransactionShape::pushdata_sizes_bytes` | Byte size for every pushdata element being checked |
| `Bip110TransactionShape::op_return_script_pubkey_sizes_bytes` | Full output ScriptPubKey bytes for every OP_RETURN output being checked |
| `Bip110TransactionShape::non_op_return_script_pubkey_sizes_bytes` | Full ScriptPubKey bytes for every non-OP_RETURN output being checked |
| `Bip110TransactionShape::witness_element_sizes_bytes` | Byte size for every witness element being checked |

`Bip110Limits::default()` and `Bip110Limits::canonical()` return the same
canonical values. `Bip110TransactionShape` is serializable/deserializable for
adapter boundaries. Its validation façade, together with
`Bip110Limits::validate_transaction`, delegates to the existing
`Bip110Compliance` aggregate validator and returns the existing structured
`Bip110ValidationResult` and `Bip110Violation` values.

The shape is transaction-wide for this subset: each vector must contain every
occurrence that the adapter is asking the core to check. The contract validates
supplied size metadata only. It does not parse raw transactions, determine
script context, perform network I/O, persist state, or enforce policy at
runtime. It is therefore a deterministic size-policy contract, not a full
consensus or script verifier.

### SDK and Gateway Adapter Mapping

Before constructing the shape, downstream adapters must inspect the transaction,
apply any BIP-110 script-context exceptions, and classify each constrained
occurrence. They should include only elements subject to the corresponding core
limit, then call `shape.validate()` or validate through an explicit
`Bip110Limits`/`Bip110Compliance` configuration. In particular, OP_RETURN
values must be the full output ScriptPubKey byte length, never payload length.

| Core contract field | `conxius-enclave-sdk` / wallet mapping | `conxian-gateway` mapping |
|---------------------|----------------------------------------|--------------------------|
| `pushdata_sizes_bytes` | Measure each script pushdata element before signing | Map observed or constructed pushdata element sizes before routing |
| `op_return_script_pubkey_sizes_bytes` | Measure the full output ScriptPubKey for every classified OP_RETURN output | Map the full output ScriptPubKey size for every classified OP_RETURN output |
| `non_op_return_script_pubkey_sizes_bytes` | Measure the full ScriptPubKey for every classified non-OP_RETURN output | Map every classified non-OP_RETURN output ScriptPubKey size |
| `witness_element_sizes_bytes` | Measure each witness stack element before signing | Map witness stack sizes from the constructed or observed transaction |

No SDK or Gateway implementation, dependency, or release is changed by this
core contract update. The mapping above is an integration contract for future
downstream adoption and does not claim unreleased SDK behavior is available.

## Conxian's Position on BIP-110

### Core Principles Alignment

| BIP-110 Goal | Conxian Implementation |
|--------------|----------------------|
| Peer-to-peer cash | Non-custodial wallet architecture |
| Decentralization | Multi-layer, non-custodial design |
| Sound money | Trust tier enforcement (CON-791) |
| Low node costs | Hardware-enclave signing optimization |
| Monetary focus | Protocol primitives over data storage |

### Why Conxian Supports BIP-110

1. **Sovereign Ownership**: Users should own their Bitcoin, not have it polluted by inscriptions
2. **Hardware Security**: Hardware wallets work better on clean chains with predictable fees
3. **Trust Tiers Matter**: BIP-110 protects T1 (Strict) trust tier for Bitcoin bridges
4. **Layer 2 Economics**: Lightning, RGB, BitVM2 work better with predictable fees
5. **Original Vision**: Bitcoin is peer-to-peer electronic cash, not a distributed database

## Repository-Specific Alignment

### 1. conxius-enclave-sdk (HIGH Priority)

**Current Bitcoin Integration:**
- `bitcoin.rs` - Bitcoin transaction handling
- `bip322.rs` - Simple proof of funds
- `bitvm2.rs` - BitVM2 optimistic verification
- `lightning.rs` - Lightning Network
- `musig2.rs` - MuSig2 key aggregation (BIP-327)

**BIP-110 Actions:**
- [ ] Add `bip110_compliant` feature flag
- [ ] Verify BIP-322 works under new rules
- [ ] Optimize fee estimation for clean blocks
- [ ] Document max data sizes in transaction builders

### 2. conxius-wallet (HIGH Priority)

**Current Bitcoin Integration:**
- Bitcoin L1 direct support
- Wormhole/NTT interlayer execution
- Android-first, offline-first
- Hardware security module integration

**BIP-110 Actions:**
- [ ] Update fee estimation models for cleaner market
- [ ] Document BIP-110 compliance in wallet operations
- [ ] Verify Silent Payments (BIP-352) compatibility
- [ ] Add inscription filtering for fee estimates

### 3. conxian-nexus (MEDIUM Priority)

**Current Bitcoin Integration:**
- "Glass Node" - observation and verification
- BTC header-chain query
- State synchronization

**BIP-110 Actions:**
- [ ] Verify lighter node sync with limited data
- [ ] Add BIP-110 compliance metrics
- [ ] Document clean state verification benefits

### 4. lib-conxian-core (MEDIUM Priority)

**Current Bitcoin Integration:**
- Chain adapters for Bitcoin
- Control models with TrustTier
- State root persistence

**BIP-110 Actions (core source tree; not a release claim):**
- [x] Keep `Bip110Compliance` as the existing validation engine
- [x] Add the additive `Bip110Limits` and `Bip110TransactionShape` core contract
- [ ] Add BIP-110 rule validation to anchoring

## Implementation Checklist

### For All Repositories

```markdown
## BIP-110 Compliance Checklist

### Transaction Building
- [ ] OP_RETURN output ScriptPubKey limited to 83 bytes (full output size, not payload)
- [ ] ScriptPubKey limited to 34 bytes (non-OP_RETURN)
- [ ] Pushdata/witness limited to 256 bytes
- [ ] No Tapleaf abuse for data embedding

### Fee Estimation
- [ ] Filter inscription spam from fee estimation
- [ ] Use clean block data for predictions
- [ ] Document fee model assumptions

### Testing
- [ ] BIP-110 rule unit tests
- [ ] Compliance integration tests
- [ ] Fee estimation accuracy tests

### Documentation
- [ ] BIP-110 compliance statement
- [ ] Max data size documentation
- [ ] Feature flag `bip110_compliant`
```

## Cross-Repository Dependencies

```
conxius-enclave-sdk
    └── lib-conxian-core (control_model, adapters)
            └── conxian-nexus (observation)
                    └── conxian-gateway (routing)

conxius-wallet
    └── conxius-enclave-sdk (signing)
            └── lib-conxian-core (types)
```

## Future Considerations

### If BIP-110 Activates

1. **Fee Market Normalization**: Cleaner fee estimation across all layers
2. **BitVM2 Efficiency**: More predictable proof verification costs
3. **Lightning Reliability**: Better channel state estimation
4. **RGB Scalability**: Cleaner base layer for RGB state

### If BIP-110 Does Not Activate

1. **Inscription Filtering**: Implement client-side filtering
2. **Fee Oracle Updates**: Continue improving spam-resistant models
3. **Alternative Layer**: Consider dedicated data layer for non-monetary use

## References

- [BIP-110 Specification](https://github.com/bitcoin/bips/blob/master/bip-0110.mediawiki)
- [BIPs.dev Explanation](https://bips.dev/110)
- [Conxian Unified Theory](docs/CONXIAN_UNIFIED_THEORY_v2.md)
- [Trust Tier (CON-791)](src/control_model/)

## Contact

- **Support**: support@conxian-labs.com
- **Security**: security@conxian-labs.com
- **Labs**: https://www.conxian-labs.com
