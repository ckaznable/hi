## ADDED Requirements

### Requirement: Cron-based scheduling
The system SHALL support cron-expression-based scheduled tasks, each executing a prompt using a specified model.

#### Scenario: Scheduled task triggers
- **WHEN** a scheduled task's cron expression matches the current time
- **THEN** the system SHALL execute the task's prompt using the specified model

#### Scenario: Multiple scheduled tasks
- **WHEN** the config contains multiple schedule entries
- **THEN** the system SHALL run each independently on its own schedule

### Requirement: Per-task model selection
Each scheduled task SHALL support an optional `model` field to specify which model to use.

#### Scenario: Task with small model
- **WHEN** a scheduled task has `model: "small"`
- **THEN** the system SHALL use the `small_model` for that task

#### Scenario: Task with default model
- **WHEN** a scheduled task omits the `model` field
- **THEN** the system SHALL use the main (default) model

#### Scenario: Task with inline custom model
- **WHEN** a scheduled task has `model: { "provider": "ollama", "model": "qwen2.5:3b" }`
- **THEN** the system SHALL use the inline model config for that task

### Requirement: Scheduler result reporting
Each scheduled task result SHALL be reported back to the main system via a channel.

#### Scenario: Task result received
- **WHEN** a scheduled task completes
- **THEN** the system SHALL send the task name and result through a channel to the main event loop

### Requirement: No schedules configured
- **WHEN** the config has an empty or missing `schedules` array
- **THEN** the system SHALL NOT start any scheduler background tasks

#### Scenario: Empty schedules
- **WHEN** `schedules` is `[]`
- **THEN** no scheduler tasks SHALL be created
