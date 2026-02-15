## Why

Currently, the application lacks a structured logging system. Errors and important events are logged inconsistently (using `eprintln!`, `println!`, or not at all), making debugging and production monitoring difficult. A dedicated logging crate with file output for error-level and above will provide consistent, persistent log records for troubleshooting and audit purposes.

## What Changes

- Add a new `log` crate to the workspace for centralized logging
- Configure log levels: error, warn to file; info, debug, trace to console (configurable)
- Route `error` level and above to a log file with rotation
- Integrate logging across all crates in the workspace
- Remove ad-hoc `eprintln!` and `println!` usage for errors, replace with proper log macros

## Capabilities

### New Capabilities
- `logging-infrastructure`: Centralized logging with file output for error-level and above, supporting configurable levels, formats, and file rotation

### Modified Capabilities
- (none - this is a new capability)

## Impact

- **New dependency**: Add `tracing` or `log` crate to `Cargo.toml`
- **Code changes**: Update error handling in `hi-core`, `hi-remote`, and other crates to use the new logging macros
- **Config**: Add logging configuration to `shared` config module
- **Runtime**: Log files stored in application data directory
