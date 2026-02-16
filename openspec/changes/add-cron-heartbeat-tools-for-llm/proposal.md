## Why

LLM-driven automation currently lacks direct tool support to manage cron schedules and edit heartbeat content, forcing manual intervention for common operational updates. Adding explicit management tools now unblocks reliable self-service scheduling and heartbeat maintenance workflows.

## What Changes

- Add tool capabilities for creating cron entries from LLM actions.
- Add tool capabilities for deleting existing cron entries from LLM actions.
- Add tool capabilities for editing heartbeat content from LLM actions.
- Define validation and error behavior for malformed cron expressions, missing targets, and edit conflicts.

## Capabilities

### New Capabilities
- `llm-cron-management-tools`: Tool-level requirements for adding and deleting cron schedules through LLM-invoked operations.
- `llm-heartbeat-content-editing`: Tool-level requirements for updating heartbeat file/content through LLM-invoked operations.

### Modified Capabilities
- None.

## Impact

- Affected systems: tool invocation layer, scheduling/cron persistence path, heartbeat content storage path.
- Affected code likely in crates handling tools and runtime coordination (e.g., `hi-tools`, `hi-core`, and related command adapters).
- API/behavior impact: expands available tool surface for LLM actions; no breaking user-facing CLI command changes expected.
