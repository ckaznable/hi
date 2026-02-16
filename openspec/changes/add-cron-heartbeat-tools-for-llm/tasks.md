## 1. Tool Surface and Contracts

- [x] 1.1 Define tool input/output schemas for cron add, cron remove, and heartbeat edit operations in the existing tool registration layer
- [x] 1.2 Register the three new tools in the runtime tool catalog with clear descriptions and parameter validation hooks
- [x] 1.3 Ensure tool responses are structured and machine-actionable for success and failure outcomes

## 2. Cron Add/Remove Implementation

- [x] 2.1 Implement cron add handler that validates required fields (`name`, `cron`, `prompt`, optional `model`) before mutation
- [x] 2.2 Enforce duplicate-name rejection for cron add and return explicit conflict errors
- [x] 2.3 Implement cron remove handler using schedule `name` as the stable identifier
- [x] 2.4 Return not-found errors for remove requests targeting missing schedules without mutating storage
- [x] 2.5 Persist successful add/remove mutations through the existing schedule storage path used by runtime scheduling

## 3. Heartbeat Content Editing Implementation

- [x] 3.1 Implement heartbeat edit handler for managed heartbeat content updates (full replacement in v1)
- [x] 3.2 Validate heartbeat edit payload completeness and reject empty or malformed requests
- [x] 3.3 Enforce managed-path-only writes so heartbeat edits cannot target arbitrary filesystem paths
- [x] 3.4 Return actionable error payloads for write failures or edit conflicts

## 4. Validation and Persistence Consistency

- [x] 4.1 Reuse or align validation rules with existing schedule/heartbeat command paths to avoid behavior drift
- [x] 4.2 Ensure read-modify-write mutation flow avoids partial state on validation or persistence failure
- [x] 4.3 Verify mutation durability by confirming runtime reload/restart reflects latest successful writes

## 5. Tests and Verification

- [x] 5.1 Add unit tests for cron add success, invalid cron rejection, and duplicate-name rejection
- [x] 5.2 Add unit tests for cron remove success and missing-name not-found behavior
- [x] 5.3 Add unit tests for heartbeat edit success, empty/invalid payload rejection, and managed-path enforcement
- [x] 5.4 Add tests for structured tool response payloads across representative failure cases
- [x] 5.5 Run package-scoped checks/tests for touched crates, then run workspace-level checks/tests before handoff
