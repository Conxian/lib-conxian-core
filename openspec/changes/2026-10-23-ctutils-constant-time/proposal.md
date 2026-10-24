# Patch constant-time utility operations

Owner: Conxian/conxian-business#1390; Core release candidate #489.

## Problem and selected option

RUSTSEC-2026-0385, published 23 October 2026, identifies non-constant-time
mask generation and u128 comparisons before ctutils 0.4.4. Core's current
locked graph resolves 0.4.2 and its hosted audit fails on this advisory.

Use Cargo's targeted compatible registry patch to 0.4.4. The upstream fix
uses an optimization barrier for masks and a 32-bit comparison special case;
upstream records BINSEC verification. Application tests alone do not prove
constant-time machine code on every target. Avoid a local cryptographic fork,
major dependency migration or audit exception: the maintained patch preserves
the API, dependency count and existing feature boundaries with no service/cost.

Primary sources inspected 23 October 2026:
- https://github.com/RustSec/advisory-db/blob/main/crates/ctutils/RUSTSEC-2026-0385.md
- https://github.com/RustCrypto/utils/pull/1569
- https://github.com/rust-lang/crates.io-index/blob/master/ct/ut/ctutils

Registry metadata: ctutils 0.4.4, published 2026-10-23T01:04:28Z, MSRV 1.87
(below repository Rust 1.98.1), not yanked. Retains cmov ^0.5.3 and optional
subtle ^2.6; existing resolutions satisfy both. Downloaded crate SHA256:
12218662ccac37d9714f2191252553d67585b97df8c839cbe49e0bde278dc865.

## Acceptance and boundaries

Cargo.lock changes only ctutils version/checksum; all other packages, manifests,
pins, workflows, validators, scripts and acceptance controls remain unchanged.
Run locked workspace tests, strict all-target/all-feature Clippy, existing audit
and boundary checks. Fresh hosted checks and genuine independent review are
required before integration. Do not mutate #489's immutable head. After ordinary
dev integration, reconcile target ancestry and generate a fresh promotion through
staged/soak/main with candidate-specific artifact/consumer/release acceptance.
Neither an audit pass nor this dependency patch qualifies signing or hardware.
