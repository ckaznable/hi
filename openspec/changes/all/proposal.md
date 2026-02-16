## Why

This repository currently has many active, non-archived OpenSpec changes that touch overlapping runtime areas (CLI entrypoints, config, remote mode, scheduling, logging, and tooling). A coordinated rollout plan is needed so implementation can proceed in a single, dependency-aware wave instead of fragmented, potentially conflicting changes.

## What Changes

- Introduce an umbrella implementation plan that consolidates active OpenSpec deltas into one execution stream.
- Define cross-change dependency ordering so implementation can proceed safely without circular or conflicting edits.
- Establish shared acceptance criteria for integration points (config loading, command routing, heartbeat/scheduler behavior, Telegram remote paths, and logging behavior).
- Define rollout and verification checkpoints for package-scoped and workspace-wide validation.

## Capabilities

### New Capabilities
- `change-rollup-orchestration`: Coordinate implementation of multiple active OpenSpec changes under one ordered plan with explicit dependency and validation gates.

### Modified Capabilities
- None.

## Impact

- Affected code: likely spans `src/main.rs`, `package/shared`, `package/hi-core`, `package/hi-remote`, `package/hi-tools`, and related tests as individual sub-changes are implemented.
- Affected APIs: no new external network API surface; CLI and runtime behavior may change as scoped sub-changes land.
- Dependencies/systems: OpenSpec workflow artifacts, workspace build/test pipeline, and cross-crate integration boundaries.
