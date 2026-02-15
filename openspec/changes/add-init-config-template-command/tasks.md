## 1. CLI command surface

- [x] 1.1 Add `Init` subcommand to `Commands` enum in `bin/hi/src/main.rs` and wire dispatch to an init handler
- [x] 1.2 Add CLI tests in `bin/hi/src/main.rs` to validate parsing for `hi init`
- [x] 1.3 Ensure help/usage text clearly describes that `init` creates a config template at default config path

## 2. Config template generation

- [x] 2.1 Add a shared helper to resolve `config_dir()/config.json` target path and create parent directory when missing
- [x] 2.2 Implement template writer logic that writes valid minimal JSON (`provider`, `model`, `api_key`, `context_window`)
- [x] 2.3 Implement existing-file guard so `init` fails without overwriting when target config already exists

## 3. Error handling and user feedback

- [x] 3.1 Return success output that includes the resolved config path when template creation succeeds
- [x] 3.2 Return actionable errors for existing-file condition and filesystem failures (permission/path issues)
- [x] 3.3 Add unit tests for success path, existing-file refusal, and write failure handling

## 4. Documentation and verification

- [x] 4.1 Update `README.md` command documentation to include `hi init` usage in setup flow
- [x] 4.2 Run `cargo check --workspace` and resolve any issues introduced by this change
- [x] 4.3 Run `cargo test --workspace` and ensure relevant tests pass
