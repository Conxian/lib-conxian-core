# Core source capability guidance

Read `../docs/CAPABILITY_CONTRACT.md` before changing capability declarations. Preserve Bitcoin L1 as the trust/finality root for universal-chain service and the zero-native-token boundary. Known identifiers, SDK modules, runtime registrations, healthy observations, and production qualification are separate facts.

Keep Core deterministic and offline. Preserve exact chain/network/operation identity, version checks, bounded freshness, unknown-field rejection, and independent evidence axes. Structural manifest validation does not authenticate a provider or authorize signing. Network/provider verification and protected execution belong to downstream services and the Enclave SDK.
