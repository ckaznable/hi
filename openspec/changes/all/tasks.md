## 1. Active Change Inventory and Dependency Mapping

- [x] 1.1 Enumerate all non-archived changes under `openspec/changes/` and capture their primary touched modules.
- [x] 1.2 Build a prerequisite map for each change (required upstream crates/behaviors before implementation).
- [x] 1.3 Validate dependency order by checking no change is scheduled before its prerequisites.

## 2. Wave Planning and Execution Gates

- [x] 2.1 Partition changes into Wave 0 (foundational/shared), Wave 1 (core features), Wave 2 (tool/runtime extensions), and Wave 3 (interface/adapter features).
- [x] 2.2 Define per-wave completion criteria with required package-scoped `cargo check` and `cargo test` commands.
- [x] 2.3 Add a fail-fast rule that blocks wave advancement when required verification fails.

## 3. Implementation Orchestration

- [x] 3.1 Execute Wave 0 changes (4/4 done, 125 tests): add-log-crate, save-cron-to-file, replace-print-with-log, split-telegram-long-messages
- [x] 3.2 Execute Wave 1 changes (4/4 done, 136 tests): add-line-and-utf8-indexed-file-io-tooling, heartbeat-default-20m-small-model-gate, add-telegram-user-id-auth-gate, add-safe-default-system-prompt-memory-rules
- [x] 3.3 Execute Wave 2 changes (6/6 done, 188 tests): extract-heartbeat-module-add-read-write-tools, add-markdown-hierarchical-memory-tool, add-llm-schedule-view-tool, add-mcp-stdio-http-with-mcp-json-config, add-model-pool-slash-command-and-fallback-switching, persist-heartbeat-cron-memory-index-and-minimal-runtime-context
- [x] 3.4 Execute Wave 3 changes (3/3 done, 204 tests): add-telegram-slash-commands-for-cron-heartbeat-management, add-skills-slash-command-and-prompt-prepend, add-main-guided-initial-setup-command

## 4. Final Integration and Sign-off

- [x] 4.1 Run `cargo check --workspace` after all waves complete — clean, 0 warnings.
- [x] 4.2 Run `cargo test --workspace` — 204 tests, 0 failures, 0 warnings.
- [x] 4.3 Skipped change: implement-aieos (no description/scope available, user chose to skip).
- [x] 4.4 All 17 active changes implemented successfully across 4 waves.
