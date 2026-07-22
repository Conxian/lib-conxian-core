# Core verifier inventory

This inventory records what `lib-conxian-core` can establish authoritatively.
Core owns protocol types, deterministic invariants, and a small number of
audited cryptographic primitives. It does not acquire chain evidence, access a
node, persist observations, or provide hardware-backed verification.

## State and chain evidence

| API | Authoritative status | Supported evidence in Core | Fail-closed outcome |
| --- | --- | --- | --- |
| `UniversalChainAdapter::verify_state_proof` for Bitcoin, EVM, Cosmos, Solana, Move, and Substrate | Not implemented | None; no light client, ZK verifier, or chain provider is linked | `StateProofError::MalformedInput` for empty input, otherwise `StateProofError::Unsupported` |
| `UniversalChainAdapter::get_state_root` for those chains | Not implemented | None; static roots are not evidence | `StateProofError::Unavailable` |
| `BabylonAdapter::verify_state_proof` | Not implemented | No BTC header/EOTS/checkpoint verifier | `StateProofError::MalformedInput` or `StateProofError::Unsupported` |
| `LiquidAdapter::verify_state_proof` | Not implemented | No Elements/Merkle/confidential/federation verifier | `StateProofError::MalformedInput` or `StateProofError::Unsupported` |
| `BabylonAdapter::get_state_root` and `LiquidAdapter::get_state_root` | Not implemented | No verified source | `StateProofError::Unavailable` |
| `StacksNakamoto::verify_bitcoin_finality_checked` | Not implemented | A block number is not Bitcoin header/transaction evidence | `StacksError::MalformedFinalityEvidence` or `StacksError::UnsupportedFinalityEvidence` |
| `SBTCBridge::get_status` | Not authoritative | No persisted/provider-backed intent lookup | `StacksError::UnknownIntent` or `StacksError::StatusUnavailable` |

The deprecated `StacksNakamoto::verify_bitcoin_finality` wrapper returns
`false`. No state or finality API in this table returns `Ok(true)` without a
real verifier.

## Protocol and message verification

| API | Authoritative status | Supported evidence in Core | Fail-closed outcome |
| --- | --- | --- | --- |
| `Bip322Bridge::verify_message_checked` | Structural only | Bitcoin address parsing and canonical base64/witness decoding | `MalformedAddress`, `MalformedSignature`, or `Unsupported`; no script execution or signature check |
| `DlcManager::verify_oracle_attestation` | Authoritative primitive | Secp256k1 point equation for the exact supplied `(oracle key, nonce point, outcome message, scalar)` tuple | `false` for malformed/equation-invalid inputs |
| `DlcManager::verify_oracle_attestation_for_intent` | Not an authorization result | It can check the real point equation plus oracle-key, outcome-hash, positive-collateral, and expiry policy, but the existing oracle tuple does not sign collateral/expiry/full-intent context | Typed `DlcVerificationError::UnsupportedIntentBinding` for an otherwise valid tuple; malformed, mismatched, expired, and invalid evidence remain typed failures |
| `DlcManager::verify_execution_checked` | Not implemented | Its compatibility arguments omit nonce, outcome, expiry height, CET, and transaction binding | `MalformedAttestation` for short input, otherwise `UnsupportedExecutionContext` |
| `FrostManager::generate_shares`, `prepare_distribution_shares`, and `aggregate_signature` | Not implemented | No audited FROST DKG, distribution, nonce, commitment, or Schnorr provider | Typed `FrostError`; no fabricated shares, encrypted payloads, or signatures |
| `FedimintAdapter::verify_unblinded_checked` | Deterministic primitive only | Exact 33-byte compressed point, 32-byte blinding factor, and real secp256k1 point reconstruction/equality; no provider-backed mint or note authorization | Typed malformed-input error or `Ok(false)` for a valid but mismatched point |
| `FedimintAdapter::get_mint_status` | Provider-backed status unavailable | Core validates only a non-empty/non-whitespace ID; it does not query or authenticate a mint or know community/liquidity data | `FedimintError::MalformedMintId` or `FedimintError::StatusUnavailable` |

The deprecated `Bip322Bridge::verify_message`, `DlcManager::verify_execution`,
and `FedimintAdapter::verify_unblinded` wrappers return `false` on every
unsupported or malformed path. `FedimintAdapter::blind_note` and
`verify_unblinded_checked` are deterministic point-reconstruction primitives,
not provider-backed mint, note, or status verification. `get_mint_status`
returns `StatusUnavailable` for every non-empty ID until an authenticated
provider is supplied; it never fabricates community or liquidity data.

## Lightning provisioning

| API | Authoritative status | Supported evidence in Core | Fail-closed outcome |
| --- | --- | --- | --- |
| `LightningNode::request_jit_channel` | Provisioning unavailable in Core | Core validates the node public-key shape only; it has no LSPS2 transport, peer/channel state, liquidity reservation, or authoritative provisioning backend | `LightningError::JITProvisioningFailed` for malformed keys; `LightningError::JitProvisioningUnavailable` for valid keys; never `Ok(true)` |

A valid node public key establishes only input shape, not a provisioned
channel. Actual JIT provisioning, peer interaction, liquidity, and channel
state belong in Gateway/Lightning infrastructure. Core must not treat parsing
alone as authorization or provisioning success.

## Enclave and rollout boundaries

| API | Authoritative status | Supported evidence in Core | Fail-closed outcome |
| --- | --- | --- | --- |
| `HeadlessEnclave::verify_attestation_chain` | Parse-only, not authoritative | DER container parsing can distinguish malformed input from a parseable sequence | `EnclaveVerificationError::MalformedDer` or `UnsupportedProvider`; even `30 00` is unsupported |
| `ZKCompliance::verify_aml_stateless_checked` | Not implemented | None; non-empty strings are not AML proof verification | `EmptyEvidence` or `UnsupportedProvider` |
| `RGBStockAdapter` and `RGBSkeletonAdapter` transition/seal APIs | Not implemented | No schema/AluVM transition or single-use-seal provider | `RGBError::VerificationUnavailable` after malformed-input checks |
| `RGBRuntime::Shadow` | Observational only | Adapter observations may be collected but cannot authorize | `RGBError::NonAuthoritativeShadow` for a usable observation; malformed input remains an error |

The deprecated `ZKCompliance::verify_aml_stateless` wrapper returns `false`.
Provider-backed attestation, AML, RGB, chain, and FROST verification belongs
in the production SDK, Nexus, Gateway, Wallet, or another explicitly owned
downstream provider. Shadow mode is never an authorization result.

## API migration rule

Callers that need to distinguish malformed evidence, an invalid cryptographic
result, and an unavailable provider should use the checked/typed APIs listed
above. Legacy boolean wrappers remain only where removing them would create an
unnecessary compatibility break; they fail closed and are deprecated. A
successful structural parse, non-empty input, string shape, static root, or
shadow observation is not proof.
