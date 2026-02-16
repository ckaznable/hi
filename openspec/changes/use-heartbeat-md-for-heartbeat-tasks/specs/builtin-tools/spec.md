## ADDED Requirements

### Requirement: Heartbeat write tool
The system SHALL provide a dedicated heartbeat-write tool for LLMs to update `HEARTBEAT.md` task content with validated status transitions.

#### Scenario: Valid heartbeat task update
- **WHEN** the LLM calls the heartbeat-write tool with a valid task update payload
- **THEN** the tool SHALL apply the update to `HEARTBEAT.md` and return a success result

#### Scenario: Invalid status transition
- **WHEN** the LLM calls the heartbeat-write tool with an invalid transition (for example `pending` directly to `done`)
- **THEN** the tool SHALL reject the request and return a validation error without modifying `HEARTBEAT.md`

## MODIFIED Requirements

### Requirement: Tool registration
All built-in tools SHALL be registered with the rig agent via the `.tool()` builder method, making them available for LLM function calling, including the heartbeat-write tool when heartbeat task-ledger support is enabled.

#### Scenario: Agent uses tools
- **WHEN** the agent is built with all built-in tools registered
- **THEN** the LLM SHALL be able to invoke any of the registered tools during a conversation

#### Scenario: Heartbeat write tool available
- **WHEN** heartbeat task-ledger support is enabled
- **THEN** the LLM SHALL be able to invoke the heartbeat-write tool to edit heartbeat task state
