# Preview workflow condition correction

Tracks #435. GitHub rejects `secrets` in a job-level condition before a runner starts. The preview workflow now checks the existing project/event predicates at job level and checks the credential's job environment at each side-effecting step. Missing credentials skip provider actions; no credential or database URL is printed. Existing branch naming, expiry, event actions and provider inputs are preserved.

Pinned actionlint 1.7.12 reproduces both original context errors and accepts the corrected workflow. Its release archive is checked against the upstream SHA-256 digest. The new syntax workflow validates the preview workflow and itself on relevant dev/staged/main changes.

Source: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets (read 2026-10-15).

Syntax success is not evidence of cloud authentication or create/close lifecycle behavior. A configured disposable-PR run and cleanup evidence remain required before #435 can close; the earlier zero-job run's event-label discrepancy remains historical evidence to reconcile.
