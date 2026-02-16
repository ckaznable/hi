## Context

Heartbeat currently runs a periodic prompt from `heartbeat.prompt` (or default `"heartbeat check"`) and persists only `last_heartbeat_epoch` in `runtime_index.json`. The codebase already has robust markdown parsing/writing patterns in `MemoryTool` and file-backed persistence for schedules in `schedules.json`. We need heartbeat behavior that is inspectable and editable by users without changing config, while preserving the existing interval/model controls.

## Goals / Non-Goals

**Goals:**
- Make `data_dir()/HEARTBEAT.md` the source of truth for heartbeat task content.
- Define deterministic task-state transitions for periodic execution (`pending` -> `in-progress` -> `done`/`failed`).
- Keep heartbeat resilient when task file is missing, empty, or malformed.
- Preserve existing heartbeat scheduling knobs (`enabled`, `interval_secs`, `model`).
- Provide a constrained heartbeat-write tool so LLM updates are scoped to heartbeat task ledger edits.

**Non-Goals:**
- Replacing cron schedules (`schedules.json`) with heartbeat tasks.
- Adding new CLI subcommands in this change.
- Building a general-purpose workflow engine beyond heartbeat task execution.

## Decisions

### Decision 1: Introduce a dedicated `HEARTBEAT.md` file under `data_dir()`
Use `data_dir()/HEARTBEAT.md` instead of overloading `memory.md` sections.

Rationale:
- Heartbeat work items and general memory have different lifecycle semantics.
- A dedicated file allows clear documentation, safer parsing, and less coupling.

Alternatives considered:
- Reusing `memory.md` section (`# HEARTBEAT`) would avoid a new file but mixes long-lived memory with mutable task state.

### Decision 2: Use strict markdown structure with explicit status tokens
Define task entries in markdown with machine-parseable status markers and metadata timestamps.

Rationale:
- Human-editable while still deterministic for parser behavior.
- Enables idempotent re-runs and crash recovery.

Alternatives considered:
- Free-form prompt-only markdown: too ambiguous for reliable status transitions.

### Decision 3: Apply optimistic read-modify-write with single-loop ownership
Heartbeat loop owns updates to `HEARTBEAT.md`; each tick performs read -> select runnable task(s) -> write transition -> execute -> write result.

Rationale:
- Matches current single heartbeat worker model.
- Minimizes coordination complexity while preserving recoverability.

Alternatives considered:
- Multi-writer locking and external mutators with stronger lock semantics; deferred until needed.

### Decision 4: Keep runtime index as summary metadata only
Continue recording `last_heartbeat_epoch` in `runtime_index.json` and optionally cache lightweight task summary fields, but keep full task source in `HEARTBEAT.md`.

Rationale:
- Avoids duplicating source data and divergence risk.

### Decision 5: Add a dedicated heartbeat write tool instead of raw file writes
Introduce a focused tool (e.g., `heartbeat_write`) that accepts structured task update inputs and rewrites only valid sections in `HEARTBEAT.md`.

Rationale:
- Safer than exposing broad `write_file` semantics for task state mutation.
- Enables validation of status transitions and markdown structure before persistence.

Alternatives considered:
- Reusing generic `write_file` directly; rejected due to higher risk of malformed task ledger and accidental unrelated edits.

## Risks / Trade-offs

- [Malformed markdown causes skipped work] -> Mitigation: strict parser with explicit error reporting and non-destructive fallback behavior.
- [Task corruption on crash between transitions] -> Mitigation: write transition checkpoints before and after execution; preserve failed entries with error note.
- [User edits during execution lead to conflicts] -> Mitigation: define last-write-wins policy with best-effort merge boundaries and deterministic section rewrite.
- [Operational drift vs cron tasks] -> Mitigation: document clear responsibility split (cron for schedule triggers, HEARTBEAT.md for heartbeat work queue).
- [Tool misuse or invalid transitions] -> Mitigation: enforce schema validation in heartbeat-write tool and reject illegal status moves.

## Migration Plan

1. Add parser/writer utilities for `HEARTBEAT.md` and define canonical task format.
2. Update heartbeat loop to source runnable tasks from `HEARTBEAT.md` instead of static prompt text.
3. Implement and register heartbeat-write tool for LLM-driven task edits.
4. Preserve existing config fields for enablement, interval, and model resolution.
5. Keep `runtime_index.json` heartbeat timestamp update behavior.
6. Document file format and tool behavior in README.

Rollback strategy:
- Revert heartbeat task-file logic and return to static prompt behavior (`heartbeat.prompt`) while leaving `HEARTBEAT.md` ignored.

## Open Questions

- Should heartbeat process one task per tick or multiple tasks per tick with budget limits?
- Should failed tasks auto-retry with attempt count, or require manual status reset to `pending`?
- Should we expose heartbeat task summaries to TUI/remote command output in a follow-up change?
