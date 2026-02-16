## ADDED Requirements

### Requirement: HEARTBEAT task ledger file
The system SHALL use `data_dir()/HEARTBEAT.md` as the persisted source of truth for heartbeat task items and their status.

#### Scenario: Ledger file missing
- **WHEN** heartbeat starts and `HEARTBEAT.md` does not exist
- **THEN** the system SHALL create an initial valid `HEARTBEAT.md` template and continue running without crashing

#### Scenario: Ledger file present
- **WHEN** heartbeat runs with an existing `HEARTBEAT.md`
- **THEN** the system SHALL parse tasks from the file and use parsed status to determine runnable work

### Requirement: Deterministic task state transitions
The system SHALL enforce deterministic heartbeat task transitions in the ledger (`pending` -> `in-progress` -> `done` or `failed`).

#### Scenario: Start task execution
- **WHEN** heartbeat selects a runnable pending task
- **THEN** the system SHALL persist that task as `in-progress` before invoking model execution

#### Scenario: Successful execution
- **WHEN** heartbeat task execution completes successfully
- **THEN** the system SHALL persist task status as `done` with completion metadata

#### Scenario: Failed execution
- **WHEN** heartbeat task execution fails
- **THEN** the system SHALL persist task status as `failed` with error metadata and keep remaining tasks intact

### Requirement: Malformed ledger safety
The system SHALL handle malformed `HEARTBEAT.md` content safely and non-destructively.

#### Scenario: Parse error
- **WHEN** heartbeat cannot parse task entries from `HEARTBEAT.md`
- **THEN** the system SHALL report a parse failure, skip destructive rewrites, and continue heartbeat scheduling for future ticks
