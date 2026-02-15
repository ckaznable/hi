## Context

The project currently uses raw `print!`/`eprintln!` for all output. This includes:
- Error messages and warnings
- Status information
- Debug output

There's no way to:
- Control verbosity at runtime
- Redirect logs to files
- Enable/disable specific log levels in production

## Goals / Non-Goals

**Goals:**
- Add structured logging with timestamps and log levels
- Provide configurable log level via config file
- Replace all print/eprint with appropriate log macros
- Keep dependencies minimal (prefer `log` + `env_logger` over heavier alternatives)

**Non-Goals:**
- Add file-based logging (only stderr for now)
- Add structured JSON logging
- Change error handling behavior
- Add tracing/span-based logging (more complex, defer to future)

## Decisions

### 1. Logging Crate: `log` + `env_logger`
**Decision**: Use the `log` crate with `env_logger` backend.

**Rationale**:
- `log` is the de facto standard logging facade in Rust
- `env_logger` is lightweight and provides good defaults (reads from RUST_LOG env var)
- Simpler than `tracing` (which requires more setup and has more overhead)
- Widely used and battle-tested

**Alternative considered**: `tracing` - More feature-rich but steeper learning curve. Defer to future if more advanced features needed.

### 2. Log Level Configuration
**Decision**: Add `log_level` field to Config struct (in shared package).

**Rationale**:
- Consistent with existing config pattern
- Allows per-environment configuration
- Falls back to `info` (default) if not specified

### 3. Migration Strategy
**Decision**: Phased migration - one package at a time.

**Rationale**:
- Reduces risk
- Easier to test
- Allows comparing old vs new output during transition

## Risks / Trade-offs

- **Risk**: Runtime dependency issues → **Mitigation**: Add to all workspace Cargo.toml files that need logging
- **Risk**: Log output format changes may break existing scripts → **Mitigation**: Keep default env_logger format (timestamp + level + message)

## Migration Plan

1. Add `log` and `env_logger` to workspace dependencies
2. Add `log_level` config field to shared config
3. Initialize logger in main.rs before any other code
4. Migrate packages in order: shared → hi-core → hi-remote → main.rs
5. Each package: replace eprintln! with log::{error!, warn!, info!}, println! with log::info!

## Open Questions

- Should `RUST_LOG` environment variable override config file? → **Decision**: Yes, env var takes precedence for dev flexibility
