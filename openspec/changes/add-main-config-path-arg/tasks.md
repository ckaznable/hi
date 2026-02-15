## 1. CLI Parameter Wiring

- [x] 1.1 Add a shared root CLI option (e.g. `--config` / `-c`) in `bin/hi/src/main.rs` and parse it as an optional config path.
- [x] 1.2 Update root command dispatch to pass the optional config path into both `tui` and `remote` execution paths.
- [x] 1.3 Extend CLI parser tests in `bin/hi/src/main.rs` to cover parsing with and without the shared config option.

## 2. Config Loading API

- [x] 2.1 Add a path-aware config loader in `package/shared/src/config.rs` to load and validate `ModelConfig` from an explicit file path.
- [x] 2.2 Keep `ModelConfig::load()` default behavior unchanged by delegating to the existing `ProjectDirs::config_dir("hi")/config.json` location when no override is provided.
- [x] 2.3 Ensure load errors include the attempted file path for both default-path and CLI-override-path failures.

## 3. Command Integration

- [x] 3.1 Update `hi_tui::run_tui` to accept optional config path input and load config via explicit path when provided.
- [x] 3.2 Update `hi_remote::run_remote` to accept optional config path input and load config via explicit path when provided.
- [x] 3.3 Keep command behavior backward-compatible when config path is not provided.

## 4. Verification

- [x] 4.1 Add/adjust unit tests for config loading to cover explicit-path success and explicit-path missing-file error.
- [x] 4.2 Run workspace checks/tests relevant to changed crates and confirm all new/updated tests pass.
- [x] 4.3 Verify no behavioral regression for default config path startup flow.
