## ADDED Requirements

### Requirement: Log initialization at application startup
The logging system SHALL be initialized at application startup before any other operations occur.

#### Scenario: Logging initializes successfully
- **WHEN** the application starts with default configuration
- **THEN** logging is initialized and ready to accept log events

#### Scenario: Logging fails to initialize
- **WHEN** the log directory cannot be created due to permission denied
- **THEN** logging falls back to stderr-only mode and emits a warning

### Requirement: Error-level logs are written to file
The system SHALL write all log events at `error` level to a log file.

#### Scenario: Error log written to file
- **WHEN** code calls `tracing::error!("connection failed")`
- **THEN** the message is appended to the daily log file

#### Scenario: Multiple errors in same day
- **WHEN** multiple error events occur within the same day
- **THEN** each error is appended to the same log file

### Requirement: Warn-level logs are written to file
The system SHALL write all log events at `warn` level to a log file.

#### Scenario: Warning log written to file
- **WHEN** code calls `tracing::warn!("rate limit approaching")`
- **THEN** the message is appended to the daily log file

### Requirement: Info-level logs are written to stderr
The system SHALL write all log events at `info` level to stderr in development mode.

#### Scenario: Info log written to stderr
- **WHEN** code calls `tracing::info!("session started")` with default configuration
- **THEN** the message is printed to stderr

### Requirement: Debug and trace logs are available in development
The system SHALL write `debug` and `trace` level logs to stderr when `RUST_LOG=debug` or `RUST_LOG=trace` is set.

#### Scenario: Debug logging enabled
- **WHEN** environment variable `RUST_LOG=debug` is set
- **THEN** debug-level messages are written to stderr

#### Scenario: Trace logging enabled
- **WHEN** environment variable `RUST_LOG=trace` is set
- **THEN** trace-level messages are written to stderr

### Requirement: Log files are rotated daily
The system SHALL create a new log file each day and archive old files.

#### Scenario: New day creates new log file
- **WHEN** the application runs past midnight
- **THEN** a new log file is created with the current date

#### Scenario: Old log files are retained
- **WHEN** more than 7 days of log files exist
- **THEN** the oldest log files are deleted

### Requirement: Log messages include timestamp and level
Each log entry SHALL include a timestamp and severity level for debugging.

#### Scenario: Log format verification
- **WHEN** a log event is written
- **THEN** the output contains timestamp in ISO 8601 format and level (e.g., `[ERROR]`, `[WARN]`)

### Requirement: Cross-crate logging works uniformly
All crates in the workspace SHALL use the same logging configuration without additional setup.

#### Scenario: Logging from hi-core crate
- **WHEN** code in `hi-core` calls `tracing::error!("provider error")`
- **THEN** the error is written to the shared log file
