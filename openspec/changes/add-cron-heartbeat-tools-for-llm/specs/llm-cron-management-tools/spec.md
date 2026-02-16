## ADDED Requirements

### Requirement: LLM tool can add cron schedules
The system SHALL provide an LLM-invokable tool that creates a schedule entry with required fields `name`, `cron`, and `prompt`, and optional `model`.

#### Scenario: Add schedule with valid payload
- **WHEN** the tool is called with a unique `name`, a valid cron expression, and a non-empty `prompt`
- **THEN** the system persists the new schedule entry and returns a success result containing the created schedule name

### Requirement: Cron add operation validates input
The system SHALL reject add requests that do not satisfy schedule validation rules.

#### Scenario: Reject invalid cron expression
- **WHEN** the add tool is called with an invalid cron expression
- **THEN** the system returns an error result and SHALL NOT write any schedule changes to persistent storage

#### Scenario: Reject duplicate schedule name
- **WHEN** the add tool is called with a `name` that already exists
- **THEN** the system returns an error indicating name conflict and SHALL NOT create a second entry with the same name

### Requirement: LLM tool can remove cron schedules
The system SHALL provide an LLM-invokable tool that removes a persisted schedule by `name`.

#### Scenario: Remove existing schedule by name
- **WHEN** the remove tool is called with a schedule `name` that exists
- **THEN** the system removes that schedule from persistent storage and returns a success result

#### Scenario: Remove fails for missing schedule
- **WHEN** the remove tool is called with a schedule `name` that does not exist
- **THEN** the system returns a not-found error and SHALL NOT modify persistent storage

### Requirement: Cron mutations are durable and observable
The system SHALL persist successful add/remove cron mutations to the runtime schedule storage used by scheduling features.

#### Scenario: Restart after mutation keeps latest schedules
- **WHEN** a schedule is added or removed successfully and the process is restarted
- **THEN** the runtime loads schedules that reflect the latest successful mutation
