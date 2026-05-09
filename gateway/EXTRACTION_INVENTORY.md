# Gateway extraction inventory

## Purpose

This inventory breaks the `gateway/` subtree into concrete move/split categories so extraction can proceed in small reviewable steps.

## Classification labels

- **move**: belongs in `conxian-gateway`
- **split**: contains mixed concerns and should be separated before moving
- **keep**: only if a reusable shared abstraction remains after cleanup

## Current file-level classification

### Runtime entrypoints

- `gateway/src/main.rs` -> **move**
- `gateway/src/mcp_server.rs` -> **move**

Reason:
- runtime entrypoints and server surfaces belong to the canonical gateway repo, not shared core

### API surface

- `gateway/src/api/mcp_handler.rs` -> **move**
- `gateway/src/api/tests.rs` -> **move**
- `gateway/src/api/mod.rs` -> **split**

Reason:
- handler and tests are runtime/service concerns
- `mod.rs` may retain only shared declarations if any are reusable after cleanup

### Engine/runtime modules

- `gateway/src/engine/mcp.rs` -> **move**
- `gateway/src/engine/support.rs` -> **move**
- `gateway/src/engine/remediation.rs` -> **move**
- `gateway/src/engine/mod.rs` -> **split**

Reason:
- engine and remediation behavior are runtime concerns
- `mod.rs` may need to be split if it exposes reusable interfaces, otherwise it should move

### Package and repo-level runtime files

- `gateway/Cargo.toml` -> **move**
- `gateway/Cargo.lock` -> **move**

Reason:
- gateway-specific package manifests should live with the gateway runtime

### Infrastructure artifacts

- `gateway/infrastructure/gcp/deployment.yaml` -> **move**

Reason:
- deployment/runtime infrastructure belongs with the runtime owner

### Gateway library surface

- `gateway/src/lib.rs` -> **split**

Reason:
- likely mixed: may expose reusable abstractions alongside runtime-oriented implementation wiring
- shared interfaces or types can remain in core only if extracted cleanly and broadly reusable

## Working rule

During extraction:
- move runtime-first files first
- split mixed files only after target interfaces are identified
- keep only shared abstractions in core

## Suggested PR sequence

### PR 1

- move runtime entrypoints and explicit server files

### PR 2

- move API handler implementation and tests

### PR 3

- move engine/runtime implementation files

### PR 4

- split and simplify `gateway/src/lib.rs`, `gateway/src/api/mod.rs`, and `gateway/src/engine/mod.rs`

### PR 5

- move package and infrastructure artifacts

## Target outcome

After extraction:
- `lib-conxian-core` keeps only shared reusable abstractions
- `conxian-gateway` clearly owns gateway runtime and adapter implementation
