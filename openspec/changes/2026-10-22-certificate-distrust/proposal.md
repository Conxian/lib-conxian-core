# Restore certificate distrust protections in the locked graph

Owner: Conxian/conxian-business#1390. Release/lineage follow-through:
Conxian/conxian-business#1415 and Core #483. This is ordinary work into dev;
it does not mutate the immutable/downward candidate or repair its ancestry.

## Verified problem and selected option

RustSec RUSTSEC-2026-0382 (2026-10-20) identifies rustls-webpki versions
before 0.103.16 as accepting paths containing distrusted emSign intermediate
public keys. RUSTSEC-2026-0383 (2026-10-19) identifies the bundled root in
webpki-root-certs before 1.0.10. Current dev resolves 0.103.15 and 1.0.9.

Primary sources, read 2026-10-22:
- https://github.com/RustSec/advisory-db/blob/main/crates/rustls-webpki/RUSTSEC-2026-0382.md
- https://github.com/RustSec/advisory-db/blob/main/crates/webpki-root-certs/RUSTSEC-2026-0383.md
- https://github.com/rustls/webpki/pull/546
- https://github.com/rustls/webpki-roots/pull/133

Select the compatible registry patch releases 0.103.16 and 1.0.10 using Cargo's
targeted update. This retains the SDK revision, manifests, feature contract and
all unrelated packages. It adds no dependency, service or resource commitment.
A prerelease WebPKI migration adds compatibility risk without a demonstrated
need; a local verifier fork adds security maintenance burden; an audit exception
does not fix the vulnerable path. Keep the normal audit active.

## Acceptance and boundaries

- Cargo.lock changes only the two package versions and registry checksums.
- Locked workspace tests and strict all-target/all-feature Clippy pass on
  Rust 1.98.1; formatting and contamination checks pass.
- Current hosted audit and default/all-feature/native compatibility lanes must
  complete on the published head, with genuine independent review.
- Keep workflows, promotion validators, policy tests, CODEOWNERS, package
  scripts, release requirements and trust/security boundaries unchanged.
- Carry accepted source through the normal dev/staged/soak/main path with real
  artifact/consumer acceptance. Source validation is not production or hardware
  qualification, an ancestry repair, or a release sign-off.

The upstream verifier contains the certificate rejection implementation and
regressions; do not replace it with an application-level metadata denylist.
