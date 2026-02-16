## Context

The runtime already supports schedule persistence (`schedules.json`) and heartbeat task updates (`heartbeat_write`), but current control surfaces are either static config or limited command flows. This change introduces LLM-invoked tools that can mutate these runtime-managed resources directly, which spans tool registration, payload validation, persistence updates, and operator-safe error reporting.

Constraints:
- Keep behavior consistent with existing schedule and heartbeat data formats.
- Keep mutations scoped to managed files under `data_dir()`.
- Avoid introducing breaking changes to existing CLI or Telegram command contracts.

## Goals / Non-Goals

**Goals:**
- Provide a tool interface to add cron schedules with full field validation and duplicate-name handling.
- Provide a tool interface to remove cron schedules by stable identifier (name).
- Provide a tool interface to edit heartbeat content in a controlled way (replace or update text content).
- Ensure mutations are durable (persisted to file) and return clear success/error payloads suitable for LLM tool loops.

**Non-Goals:**
- Reworking scheduler internals or cron execution semantics.
- Designing a new heartbeat task format beyond the currently supported markdown/task model.
- Adding unrelated general-purpose file editing tools.

## Decisions

1. **Add dedicated domain tools instead of overloading existing generic write capabilities**
   - **Decision:** Introduce explicit tools for cron add/remove and heartbeat edit operations.
   - **Rationale:** Narrowly scoped tools reduce accidental destructive writes and allow stronger domain validation before persistence.
   - **Alternative considered:** Reuse a generic file write tool for schedules/heartbeat. Rejected because validation and safety guarantees become weak and error-prone.

2. **Use name-based identity for cron deletion**
   - **Decision:** Delete schedule entries by `name` (the existing user-facing identity in schedule commands and persistence).
   - **Rationale:** Matches existing UX and avoids introducing new IDs/migration complexity.
   - **Alternative considered:** Index-based deletion. Rejected due to instability under concurrent modifications and poor ergonomics.

3. **Perform strict input validation before writes**
   - **Decision:** Validate cron expression shape, required prompt/name fields, and heartbeat edit payload completeness before file mutation.
   - **Rationale:** Fail-fast behavior prevents partial/bad state in persisted files and produces deterministic tool outcomes.
   - **Alternative considered:** Best-effort writes with downstream parse failure handling. Rejected due to hidden corruption risk.

4. **Persist through existing storage boundaries**
   - **Decision:** Route writes through existing schedule/heartbeat persistence paths used by runtime flows.
   - **Rationale:** Reuses tested serialization/format behavior and keeps source-of-truth logic centralized.
   - **Alternative considered:** New ad-hoc file writers per tool. Rejected as duplicate logic and drift risk.

5. **Return structured tool responses with actionable errors**
   - **Decision:** Return machine-readable success/error messages including failure reasons (e.g., duplicate schedule name, missing target, invalid cron).
   - **Rationale:** LLM tool orchestration depends on clear error semantics to self-correct.
   - **Alternative considered:** Plain text only responses. Rejected because they are harder for automated retries to parse reliably.

## Risks / Trade-offs

- **[Risk] Concurrent writes to schedules/heartbeat files** -> **Mitigation:** Reuse existing single-process runtime write paths and perform read-modify-write atomically within tool execution boundaries.
- **[Risk] LLM-issued destructive edits** -> **Mitigation:** Keep tools narrowly scoped (add/remove cron, bounded heartbeat edit operations) and require explicit targets.
- **[Risk] Validation mismatch with existing command paths** -> **Mitigation:** Share validation helpers or mirror command-path rules to guarantee consistent acceptance/rejection behavior.
- **[Trade-off] More tool surface area increases maintenance** -> **Mitigation:** Keep interfaces minimal and aligned with current domain models.

## Migration Plan

1. Add tool definitions and handlers in the existing tool registry path.
2. Wire handlers to existing schedule and heartbeat persistence mechanisms.
3. Add/extend tests for validation, mutation success, and failure paths.
4. Verify runtime behavior with existing schedule-view and heartbeat flows.
5. Rollback strategy: disable/unregister newly added tools and keep existing command/config flows unchanged.

## Open Questions

- Should heartbeat editing be full-document replacement only, or also support targeted task-level patch operations in v1?
- Should cron add support an optional overwrite flag when schedule name already exists, or always reject duplicates?
- Do we require audit logging for tool-triggered mutations beyond current stderr/runtime logs?
