## 1. HEARTBEAT.md Ledger Foundation

- [x] 1.1 Define canonical `HEARTBEAT.md` markdown schema (sections, task fields, status tokens, metadata)
- [x] 1.2 Implement parser + serializer utilities for heartbeat task entries with non-destructive error handling
- [x] 1.3 Add file initialization behavior that creates default `data_dir()/HEARTBEAT.md` when missing

## 2. Heartbeat Runtime Integration

- [x] 2.1 Update heartbeat loop to source runnable tasks from `HEARTBEAT.md` instead of static prompt text
- [x] 2.2 Implement deterministic transition checkpoints (`pending` -> `in-progress` -> `done`/`failed`) with persisted writes
- [x] 2.3 Keep model/interval config behavior intact and preserve `runtime_index.last_heartbeat_epoch` updates

## 3. Heartbeat Write Tool

- [x] 3.1 Add a dedicated heartbeat-write tool in `hi-tools` with structured input for task edits
- [x] 3.2 Enforce validation rules in tool (including invalid transition rejection) before mutating `HEARTBEAT.md`
- [x] 3.3 Register heartbeat-write tool in agent tool wiring and expose it for LLM invocation

## 4. Validation, Tests, and Docs

- [x] 4.1 Add unit tests for HEARTBEAT.md parse/write and malformed-file recovery paths
- [x] 4.2 Add heartbeat integration tests for task selection, success/failure transitions, and channel reporting
- [x] 4.3 Add heartbeat-write tool tests for valid updates and validation failures
- [x] 4.4 Update README with HEARTBEAT.md format, lifecycle semantics, and heartbeat-write tool usage
