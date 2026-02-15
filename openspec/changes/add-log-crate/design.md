## Context

The application currently uses `eprintln!` and `println!` for output across 5 files in the workspace:
- `src/main.rs` - CLI entry point
- `package/hi-remote/src/telegram.rs` - Telegram adapter
- `package/hi-remote/src/session_manager.rs` - Session management
- `package/shared/src/memory.rs` - Memory reclamation
- `package/hi-core/src/provider.rs` - Provider streaming

This ad-hoc approach makes it difficult to:
- Filter logs by severity level
- Persist logs to files for later analysis
- Configure output format consistently
- Rotate log files in production

The project uses Rust edition 2024, tokio async runtime, and has a modular workspace structure.

## Goals / Non-Goals

**Goals:**
- Add a logging crate to provide centralized logging infrastructure
- Route `error` and `warn` level logs to a file with rotation
- Route `info`, `debug`, and `trace` to stderr (configurable)
- Replace existing `eprintln!` calls with proper log macros
- Keep the dependency lightweight (no heavy logging frameworks)

**Non-Goals:**
- Structured logging with JSON output (future consideration)
- Logging to external services (Datadog, Sentry, etc.)
- Changing `println!` for normal user output (only error paths)
- Cross-crate log filtering at runtime (level per crate)

## Decisions

### 1. Logging crate: `tracing` over `log`

**Decision:** Use `tracing` crate instead of the simpler `log` crate.

**Rationale:**
- `tracing` provides spans (context-aware logging) which is valuable for async code
- Better integration with tokio and async contexts in this project
- Active maintenance and ecosystem support
- `tracing-subscriber` provides the file rotation and formatting we need

**Alternatives considered:**
- `log` + `simplelog`: Simpler, but less flexible for async contexts
- `tracing` only (no subscriber): Would need custom implementation for file output

### 2. Log file location: `data_dir()/logs/`

**Decision:** Store log files in the application data directory under a `logs/` subdirectory.

**Rationale:**
- Follows existing pattern: config in `config_dir()`, history in `data_dir()`
- Platform-appropriate (respects XDG on Linux, AppData on Windows)
- Easy to find for debugging

**Alternatives considered:**
- `/var/log/hi`: Requires root on Linux, not portable
- Next to binary: Not appropriate for installed applications

### 3. File rotation: Daily rotation with 7-day retention

**Decision:** Rotate log files daily, keeping 7 days of history.

**Rationale:**
- Simple to implement with `tracing-subscriber::fmt::writer::TabWriter`
- Prevents disk space exhaustion
- Sufficient for debugging production issues

**Alternatives considered:**
- Size-based rotation: More complex, may not be needed for this use case
- No rotation: Risk of unbounded disk growth

### 4. Default log level: `warn` to file, `info` to stderr

**Decision:** 
- File: receives `error` and `warn`
- Stderr: receives `info`, `debug`, `trace` (development)

**Rationale:**
- Production: error-only file logs minimize disk I/O
- Development: info+ gives useful feedback without noise
- Configurable via environment variable `RUST_LOG`

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Disk full on log write | Application crash | Wrap file write in `Result` handling, fallback to stderr |
| Log directory permission denied | No logging | Create directory with appropriate permissions, warn if fails |
| Large log files | Disk space | 7-day retention with daily rotation |
| Performance impact | Slower async I/O | Use buffered writer, async file append |
| Existing `println!` not migrated | Inconsistent logging | Document migration in tasks phase |

## Migration Plan

1. Add `tracing` and `tracing-subscriber` to workspace dependencies
2. Create logging initialization module in `shared` crate
3. Add logging config fields to `Config` struct (optional, env var for now)
4. Replace `eprintln!` calls in:
   - `src/main.rs`
   - `package/hi-remote/src/telegram.rs`
   - `package/hi-remote/src/session_manager.rs`
   - `package/shared/src/memory.rs`
   - `package/hi-core/src/provider.rs`
5. Test in development mode
6. Deploy: No migration needed (new feature only)

## Open Questions

1. **Should log level be configurable per-crate?** - Not needed initially, can add later if requested
2. **Should we log to syslog on Linux?** - Deferred, can add later for server deployments
3. **Should structured (JSON) logging be supported?** - Deferred, text format sufficient for now
