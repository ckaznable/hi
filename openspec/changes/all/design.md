## Context

The active OpenSpec queue contains multiple unarchived deltas with overlapping surface area across CLI routing, config loading, scheduling/heartbeat, logging, and Telegram remote execution paths. Executing them independently risks merge conflicts, duplicated behavior changes, and inconsistent verification depth. This design introduces an umbrella orchestration layer at planning level (not runtime code) that sequences implementation into dependency-aware waves.

## Goals / Non-Goals

**Goals:**
- Define a deterministic implementation order for active OpenSpec changes.
- Provide integration checkpoints for cross-crate seams (`shared`, `hi-core`, `hi-remote`, `hi-tools`, CLI entrypoint).
- Require consistent verification gates (package-scoped checks first, workspace checks before completion).
- Reduce risk of contradictory behavior across concurrently touching changes.

**Non-Goals:**
- Introducing a new runtime orchestrator in production code.
- Rewriting existing OpenSpec schema or CLI tooling.
- Collapsing all change documents into one mega-spec file.

## Decisions

1. **Use one umbrella capability for orchestration, keep feature-level behavior in existing changes.**
   - Rationale: Preserves traceability of original intent while adding a top-level execution contract.
   - Alternative considered: Merge all changes into a single giant capability spec. Rejected due to loss of auditability and difficult archive history.

2. **Group implementation into dependency waves.**
   - Wave 1: foundational/shared changes (config, logging, low-level tooling).
   - Wave 2: core runtime behaviors (session/model/scheduler/heartbeat).
   - Wave 3: interface adapters (TUI/remote/Telegram) and command UX.
   - Rationale: Upstream dependencies stabilize before downstream interfaces consume them.
   - Alternative considered: FIFO by creation date. Rejected because ordering by chronology does not reflect technical dependency.

3. **Apply standard validation gates per wave and at final integration.**
   - Package-scoped `cargo check`/`cargo test` while iterating.
   - Workspace-wide verification before declaring rollout complete.
   - Rationale: Contains failure blast radius and keeps feedback loops short.
   - Alternative considered: only final workspace test. Rejected due to late detection of regressions.

## Risks / Trade-offs

- **[Risk]** Umbrella plan may become stale as new changes are added. → **Mitigation**: Re-evaluate wave assignment whenever active change set changes.
- **[Risk]** Some deltas have hidden coupling not visible from artifact text. → **Mitigation**: add pre-wave dependency scan and adjust order before coding.
- **[Risk]** Larger batch implementation can extend cycle time. → **Mitigation**: keep per-wave completion criteria strict and time-boxed.
- **[Trade-off]** Additional planning overhead before coding. → **Benefit**: lower integration risk and cleaner verification boundaries.
