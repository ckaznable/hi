## Why

The heartbeat loop currently runs a static prompt, so recurring background work is hard to track, prioritize, and evolve over time. We need a file-backed workflow so heartbeat can operate against an explicit task list that users can inspect and edit.

## What Changes

- Add a heartbeat task ledger file at `data_dir()/HEARTBEAT.md` as the source of truth for pending and completed heartbeat work.
- Change heartbeat execution to read task items from `HEARTBEAT.md`, run work from that list, and persist status updates back to the same file.
- Define a stable markdown structure for task state (pending/in-progress/done), timestamps, and optional notes/errors.
- Add safety behavior for concurrent updates and partial failures so task state remains recoverable.
- Expose clear runtime behavior when `HEARTBEAT.md` is missing, empty, or malformed.
- Add a dedicated heartbeat-write tool so the LLM can update heartbeat tasks safely without arbitrary file edits.

## Capabilities

### New Capabilities
- `heartbeat-task-ledger`: File-backed heartbeat task tracking using `HEARTBEAT.md`, including task lifecycle and persistence rules.

### Modified Capabilities
- `heartbeat`: Heartbeat behavior changes from static prompt-only execution to task-ledger-driven execution with persisted state updates.
- `builtin-tools`: Add and register a heartbeat-focused write tool that edits `HEARTBEAT.md` task state.

## Impact

- Affected code: `package/hi-core/src/heartbeat.rs`, `package/hi-core/src/provider.rs`, `package/hi-tools/src/*`, `package/shared/src/runtime_index.rs` (if summary metadata is retained), and related tests.
- New runtime data artifact: `data_dir()/HEARTBEAT.md`.
- Documentation updates in `README.md` for heartbeat task file semantics.
- No API surface breakage expected for CLI subcommands, but heartbeat behavior changes at runtime.
