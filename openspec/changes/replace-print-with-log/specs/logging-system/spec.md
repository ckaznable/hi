## Summary

This specification defines the logging system for the hi project, providing structured logging with configurable log levels.

## ADDED Requirements

### Requirement: Logging system initialization
The system SHALL initialize a logger at application startup that writes to stderr with timestamps and log levels.

#### Scenario: Logger initialized with default level
- **WHEN** the application starts without RUST_LOG environment variable and no log_level in config
- **THEN** the logger SHALL use "info" as the default log level

#### Scenario: Logger initialized with environment variable
- **WHEN** the application starts with RUST_LOG environment variable set
- **THEN** the logger SHALL use the log level specified by RUST_LOG, overriding config

#### Scenario: Logger initialized with config level
- **WHEN** the application starts with log_level set in config.json
- **THEN** the logger SHALL use the configured log level (unless RUST_LOG is set)

### Requirement: Log level configuration
The system SHALL support a `log_level` configuration option in the config file.

#### Scenario: Config with valid log level
- **WHEN** config contains `"log_level": "debug"`
- **THEN** debug-level and above logs SHALL be emitted

#### Scenario: Config with invalid log level
- **WHEN** config contains an invalid log level string
- **THEN** the system SHALL fall back to "info" level with a warning

### Requirement: Structured log output
Log messages SHALL include timestamp and log level in a consistent format.

#### Scenario: Log message output
- **WHEN** code calls `log::info!("message")`
- **THEN** output SHALL include timestamp in ISO 8601 format and log level indicator (e.g., `INFO`)

### Requirement: Error logging replaces eprintln!
All error messages currently using `eprintln!` SHALL be migrated to use `log::error!` or `log::warn!`.

#### Scenario: Error logging
- **WHEN** the application encounters an error condition
- **THEN** it SHALL log at error level with appropriate context

### Requirement: Status logging replaces println!
All status messages currently using `println!` SHALL be migrated to use `log::info!`.

#### Scenario: Status logging
- **WHEN** the application completes a significant operation
- **THEN** it SHALL log at info level with appropriate message
