# Gateway extraction migration note

## Status

Planned extraction in progress

## Why this note exists

`lib-conxian-core` is being aligned to the approved builder-platform architecture.

Under that architecture:
- `lib-conxian-core` owns shared capability interfaces, shared types, and safety/verification primitives
- `conxian-gateway` owns gateway runtime, adapter implementation, API/server concerns, and deployment/runtime ownership

A substantial `gateway/` subtree currently exists in this repo. That overlap is scheduled for extraction.

## What to expect

Upcoming cleanup work will:
- move gateway runtime concerns out of this repo
- preserve only reusable shared abstractions in core
- reduce contributor confusion around core versus gateway ownership

## Working rule during migration

When editing gateway-related material in this repo:
- prefer keeping only reusable interfaces and shared primitives here
- avoid adding new runtime or adapter-specific logic here
- assume gateway runtime concerns should converge into `conxian-gateway`

## Reference

See the current extraction plan maintained in the portfolio architecture docs for the approved move/split direction.
