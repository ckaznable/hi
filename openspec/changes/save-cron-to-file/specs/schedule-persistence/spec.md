## ADDED Requirements

### Requirement: Schedules are loaded from schedules.json
The system SHALL load scheduled tasks from `data_dir()/schedules.json` when the file exists.

#### Scenario: schedules.json exists
- **WHEN** the file `schedules.json` exists in the data directory
- **THEN** the scheduler loads all tasks from this file

#### Scenario: schedules.json does not exist
- **WHEN** the file `schedules.json` does not exist
- **THEN** the scheduler falls back to loading schedules from `config.json`

### Requirement: Invalid schedules.json is handled gracefully
The system SHALL handle corrupted or invalid JSON in schedules.json without crashing.

#### Scenario: schedules.json has invalid JSON
- **WHEN** schedules.json contains invalid JSON
- **THEN** the system logs a warning and falls back to config.json schedules

#### Scenario: schedules.json is empty
- **WHEN** schedules.json is an empty file or contains only `{}`
- **THEN** the system loads no schedules (empty schedule list)

### Requirement: Schedule file can be created manually
Users SHALL be able to create schedules.json manually with the correct format.

#### Scenario: Manual schedule file creation
- **WHEN** user creates `data_dir()/schedules.json` with valid schedule entries
- **THEN** the scheduler loads and executes those schedules on next startup

### Requirement: Schedule entries include required fields
Each schedule entry SHALL contain `name`, `cron`, and `prompt` fields.

#### Scenario: Valid schedule entry
- **WHEN** a schedule entry contains `name`, valid `cron` expression, and `prompt`
- **THEN** the schedule is loaded successfully

#### Scenario: Missing required field
- **WHEN** a schedule entry is missing `name`, `cron`, or `prompt`
- **THEN** that entry is skipped with a warning log

### Requirement: Schedule data is written atomically
The system SHALL write schedules to file atomically to prevent data loss.

#### Scenario: Saving schedules
- **WHEN** schedules are saved to file
- **THEN** the write operation uses atomic rename to prevent partial writes
