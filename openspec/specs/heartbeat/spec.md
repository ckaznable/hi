## ADDED Requirements

### Requirement: Heartbeat interval
The system SHALL support a configurable heartbeat that periodically executes a prompt using a specified model.

#### Scenario: Heartbeat enabled
- **WHEN** the config has `heartbeat.enabled: true` and `heartbeat.interval_secs: 300`
- **THEN** the system SHALL execute the heartbeat prompt every 300 seconds using the specified model

#### Scenario: Heartbeat disabled
- **WHEN** the config has `heartbeat.enabled: false`
- **THEN** the system SHALL NOT start any heartbeat background task

#### Scenario: Heartbeat uses small model
- **WHEN** the heartbeat config has `model: "small"`
- **THEN** the system SHALL use the `small_model` for heartbeat prompts

#### Scenario: Heartbeat uses default model
- **WHEN** the heartbeat config omits the `model` field
- **THEN** the system SHALL use the main (default) model for heartbeat prompts

### Requirement: Heartbeat result reporting
The heartbeat result SHALL be reported back to the main system via a channel for logging or TUI display.

#### Scenario: Heartbeat result received
- **WHEN** a heartbeat prompt completes
- **THEN** the system SHALL send the result through a channel to the main event loop
