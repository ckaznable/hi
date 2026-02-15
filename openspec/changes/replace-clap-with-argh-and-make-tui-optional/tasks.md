## 1. Dependency and feature wiring

- [x] 1.1 Replace `clap` dependency usage for `hi-cli` with `argh` in workspace/package manifests.
- [x] 1.2 Make `hi-tui` optional in `bin/hi/Cargo.toml` and add `tui` feature with `default = []`.

## 2. CLI parser migration

- [x] 2.1 Refactor `bin/hi/src/main.rs` parser types and attributes from `clap` derive model to `argh` derive model.
- [x] 2.2 Preserve command routing for `init`, `remote`, `config validate`, and `tui` under the new parser.
- [x] 2.3 Update parser-focused unit tests to validate equivalent success and failure command parsing behavior.

## 3. Feature-aware TUI command behavior

- [x] 3.1 Add feature-gated TUI dispatch path so `hi_tui::run_tui()` is only used when `tui` feature is enabled.
- [x] 3.2 Implement non-TUI-build handling for `hi tui` with actionable remediation text and non-zero exit behavior.
- [x] 3.3 Verify `hi-cli` compiles in both modes: default features and `--features tui`.

## 4. Documentation and validation

- [x] 4.1 Update `README.md` command examples to document default build behavior and enabling TUI with `--features tui`.
- [x] 4.2 Run targeted checks/tests for `hi-cli` to confirm parser migration and feature-gated command behavior.
