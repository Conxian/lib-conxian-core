# Cross-repo crypto test vectors (canonical)

This directory is the single source of truth for the cryptographic primitives
that are implemented in more than one Conxian repository. The same vector files
MUST be validated by every repository that ships an implementation of the
corresponding primitive, so the Rust and Kotlin implementations cannot silently
diverge.

## Where the real crypto lives

| Primitive | `conxius-enclave-sdk` (Rust, server/Nitro) | `conxius-wallet` (Kotlin, client) | `lib-conxian-core` (this repo) |
| --- | --- | --- | --- |
| secp256k1 / Schnorr (BIP-340) | FROST threshold | BouncyCastle (`TaprootSigner`) | contract layer only |
| Taproot (BIP-341/86/350) | enclave signing | BouncyCastle (`TaprootSigner`) | byte-shape contract (`src/bitcoin/taproot.rs`) |
| MuSig2 (BIP-327) / FROST | FROST | BouncyCastle (`Musig2Signer`) | — |
| DLC adaptor signatures | — | BouncyCastle (`AdaptorSigner`) | **sha256 placeholder** (`src/crypto/mod.rs` `AdaptorSignature`) |
| BOLT-11 invoice signing | — | BouncyCastle (`LightningInvoiceSigner`) | — |
| BIP-322 message signing | — | TS/Kotlin (`services/signer.ts`) | witness-shape check only (`src/bitcoin/bip322.rs`) |

> **Important:** `lib-conxian-core` is a *contract/abstraction* layer, not a
> real-crypto implementation. Its `AdaptorSignature` (`src/crypto/mod.rs`) returns
> `SHA256("ADAPTOR-SIG-V1" || pubkey || msg)` — a commitment placeholder, **not** a
> Schnorr adaptor signature. It must not be mistaken for the real primitive. The
> real implementations are `conxius-enclave-sdk` (server) and `conxius-wallet`
> (client); those two are the ones that must agree on these vectors.

## Vector sources

- `bip340_schnorr.json` — BIP-340 sign vectors 0–3 (official Schnorr test vectors).
- `dlc_adaptor.json` — `discreetlogcontracts/dlcspecs` `test/dlc_schnorr_test.json`.
- `bolt11_invoice.json` — canonical BOLT-11 example invoice (`lnbc1pvjluez…`) message hash.

## Encoding

Private scalars (secret keys and nonces) are stored as JSON **byte arrays**
(`0..255` integers), not hex strings — these are public test-vector values, but
the byte-array form avoids secret-scanner false positives. Public values (public
keys, message hashes, signatures, adaptor points) are lowercase hex.

## Obligation

A change to any of these vectors, or the discovery of a new divergence, must be
raised against this file first, then propagated to both real-crypto repos.
