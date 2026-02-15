## 1. Setup Dependencies

- [ ] 1.1 Add `log` and `env_logger` dependencies to workspace root `Cargo.toml`
- [ ] 1.2 Add `log_level` field to Config struct in `package/shared/src/config.rs`

## 2. Initialize Logger

- [ ] 2.1 Add logger initialization in `src/main.rs` before any other code
- [ ] 2.2 Make logger respect RUST_LOG env variable and config log_level

## 3. Migrate shared Package

- [ ] 3.1 Add `log` dependency to `package/shared/Cargo.toml`
- [ ] 3.2 Replace eprintln! in `package/shared/src/memory.rs` with log macros

## 4. Migrate hi-core Package

- [ ] 4.1 Add `log` dependency to `package/hi-core/Cargo.toml`
- [ ] 4.2 Replace eprintln! in `package/hi-core/src/provider.rs` with log macros

## 5. Migrate hi-remote Package

- [ ] 5.1 Add `log` dependency to `package/hi-remote/Cargo.toml`
- [ ] 5.2 Replace eprintln! in `package/hi-remote/src/telegram.rs` with log macros
- [ ] 5.3 Replace eprintln! in `package/hi-remote/src/session_manager.rs` with log macros

## 6. Migrate main.rs

- [ ] 6.1 Replace println!/eprintln! in `src/main.rs` with log macros
- [ ] 6.2 Add appropriate log level annotations to each call site

## 7. Verification

- [ ] 7.1 Run `cargo check --workspace` to verify no compilation errors
- [ ] 7.2 Run `cargo test --workspace` to ensure all tests pass
- [ ] 7.3 Verify log output format matches specification
