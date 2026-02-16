## 1. Setup Dependencies

- [x] 1.1 Add `tracing` and `tracing-subscriber` to workspace dependencies in `Cargo.toml`
- [x] 1.2 Add `tracing` to all crate dependencies that need logging (hi-core, hi-remote, shared)

## 2. Create Logging Infrastructure

- [x] 2.1 Create `shared/src/logging.rs` module with initialization function
- [x] 2.2 Configure file writer with daily rotation and 7-day retention
- [x] 2.3 Configure stderr writer for info/debug/trace levels
- [x] 2.4 Export logging initialization from `shared/src/lib.rs`

## 3. Integrate Logging into Crates

- [x] 3.1 Update `src/main.rs` to initialize logging at startup
- [x] 3.2 Replace `eprintln!` in `main.rs` with `tracing::error!` / `tracing::warn!`
- [x] 3.3 Replace `eprintln!` in `package/hi-remote/src/telegram.rs` with tracing macros
- [x] 3.4 Replace `eprintln!` in `package/hi-remote/src/session_manager.rs` with tracing macros
- [x] 3.5 Replace `eprintln!` in `package/shared/src/memory.rs` with tracing macros
- [x] 3.6 Replace `eprintln!` in `package/hi-core/src/provider.rs` with tracing macros

## 4. Verify and Test

- [x] 4.1 Run `cargo check --workspace` to verify compilation
- [x] 4.2 Test that log files are created in `data_dir()/logs/`
- [x] 4.3 Test that error/warn messages appear in log file
- [x] 4.4 Test that `RUST_LOG=debug` enables debug output to stderr
- [x] 4.5 Run existing tests to ensure no regressions
