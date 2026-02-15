## Why

The codebase currently uses `print!`, `println!`, `eprint!`, and `eprintln!` for all output. This creates several issues:
- No log level control (can't silence debug/info logs in production)
- No structured timestamps or metadata
- No way to redirect logs to files or external systems
- Inconsistent output format across modules

Adopting a proper logging framework (like `log` + `env_logger` or `tracing`) provides controlled, structured logging with timestamps, levels, and flexible output options.

## What Changes

- Add `log` crate as a dependency (minimal, widely used)
- Add `env_logger` or similar for easy initialization
- Replace all `eprintln!` calls with `log::error!`, `log::warn!`, `log::info!`
- Replace all `println!` calls with `log::info!` or `log::debug!`
- Add log level configuration to `Config` struct
- Initialize logger at application startup

## Capabilities

### New Capabilities
- `logging-system`: Structured logging with configurable levels and output

### Modified Capabilities
- (none - this is an infrastructure change)

## Impact

- New dependencies: `log` crate (and one backend like `env_logger`)
- Affected files:
  - `src/main.rs` (~15 print/eprint calls)
  - `package/hi-remote/src/telegram.rs` (~5 eprintln calls)
  - `package/hi-remote/src/session_manager.rs` (~5 eprintln calls)
  - `package/shared/src/memory.rs` (~2 eprintln calls)
  - `package/hi-core/src/provider.rs` (~1 eprintln call)
- No external API changes
