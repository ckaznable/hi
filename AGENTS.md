# AGENTS.md

Repository guidance for coding agents working in `/data/repo/hi`.

## 1) Project Snapshot

- Language: Rust (workspace, edition `2024`).
- Workspace root: `Cargo.toml` (members under `package/*`).
- Primary binary crate: root package (`hi-cli`, bin name `hi`).
- Major crates:
  - `shared`: config/path types and validation.
  - `hi-history`: chat history persistence and compaction.
  - `hi-tools`: tool adapters (`bash`, `read_file`, etc.).
  - `hi-core`: chat session/agent/provider/scheduler logic.
  - `hi-tui`: interactive terminal UI.
  - `hi-remote`: Telegram remote adapter.

## 2) Source-of-Truth Files

- Commands and dev workflow: `README.md`.
- Workspace/package boundaries: all `Cargo.toml` files.
- Runtime entrypoint: `src/main.rs`.
- Config schema and validation: `package/shared/src/config.rs`.

## 3) Build / Check / Test Commands

Run from repository root unless noted.

### Core commands (explicitly documented)

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -- tui`
- `cargo run -- remote`
- `cargo install --path .`

### Useful package-scoped commands

- `cargo check -p hi-cli`
- `cargo check -p hi-core`
- `cargo test -p hi-core`
- `cargo test -p shared`
- `cargo test -p hi-history`
- `cargo test -p hi-tools`
- `cargo test -p hi-tui`
- `cargo test -p hi-remote`

### Run a single test (important)

- By test name in one package:
  - `cargo test -p hi-core test_stream_accumulation_single_buffer`
- By test name in workspace (slower):
  - `cargo test test_parse_openai_config`
- Show test output:
  - `cargo test -p hi-remote test_split_message_over_limit -- --nocapture`
- Run all tests in one module/file via name prefix:
  - `cargo test -p shared test_config_with_`

### Lint / format status

- No repo-local `rustfmt.toml`, `clippy.toml`, `Makefile`, `justfile`, or CI workflows were found.
- `clippy`/`fmt` commands are therefore inferred (not enforced by checked-in config):
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- If clippy introduces unrelated churn, scope linting to touched package first.

## 4) Command Execution Notes

- Prefer package-scoped checks/tests while iterating; run workspace-wide before finishing larger tasks.
- For behavior changes, add or update nearby unit tests in same file (`#[cfg(test)] mod tests`).
- Keep commands non-interactive and reproducible.

## 5) Code Style Conventions (Observed)

These are patterns already present in the codebase and should be followed.

### 5.1 Imports and module layout

- Import grouping pattern:
  1. `std::*`
  2. external crates (`anyhow`, `tokio`, `serde`, etc.)
  3. workspace/local crates (`shared::*`, `crate::*`, `hi_*::*`)
- Keep explicit imports; avoid wildcard/glob imports.
- Use small focused modules; expose modules in `lib.rs` via `pub mod ...`.

### 5.2 Naming

- Types/enums/traits: `CamelCase` (`ChatSession`, `ValidationErrorKind`).
- Functions/variables/modules: `snake_case` (`run_polling_loop`, `send_message_streaming`).
- Constants: `UPPER_SNAKE_CASE` (`MAX_MESSAGE_LENGTH`, `DEFAULT_COMPACT_PROMPT`).
- Keep acronym style consistent with existing identifiers (`Tui`, not all-caps type names).

### 5.3 Types and serde usage

- Prefer strongly typed config/domain structs over loose maps.
- Derive common traits where useful: `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`.
- Use serde rename attributes to lock external JSON shape:
  - `#[serde(rename_all = "lowercase")]`
  - `#[serde(rename_all = "kebab-case")]`
- Use enums for constrained values (e.g., provider/strategy variants).

### 5.4 Error handling

- Prefer `anyhow::Result<T>` for application/service boundaries.
- Add context on IO/parse failures via `Context` / `with_context`.
- Use `bail!` for validation failures and clear user-facing messages.
- In adapter/tool layers, define typed errors with `thiserror` when helpful.
- Avoid silent failure; log actionable details (`eprintln!`) when handling recoverable runtime errors.

### 5.5 Async and concurrency

- Tokio is the runtime (`#[tokio::main]`, `#[tokio::test]`).
- Use `tokio::spawn` for independent background tasks.
- Use `tokio::sync::mpsc` for streaming chunks between tasks.
- Keep async functions non-blocking; if using sync filesystem APIs, keep them in sync contexts.

### 5.6 Testing conventions

- Tests are colocated with implementation (`#[cfg(test)] mod tests`).
- Prefer many focused unit tests over one broad integration-style unit test.
- Async behavior tests use `#[tokio::test]`.
- Use `tempfile` for filesystem-dependent tests.
- `unwrap()` is common in tests; avoid it in production paths unless justified.

## 6) Change Scope Guidance

- Make minimal, targeted edits; avoid opportunistic refactors during bugfixes.
- Match existing crate boundaries and ownership (do not move logic between crates unless requested).
- Keep public API shape stable unless the task explicitly asks for API changes.
- When adding config fields, update:
  - struct definition,
  - validation logic,
  - docs/examples in `README.md`,
  - relevant tests.

## 7) Runtime and UX Patterns

- CLI routes by subcommand in `src/main.rs`.
- TUI supports `/quit`, `/exit`, `/reset`, `Esc`, `Ctrl+C`.
- Remote adapter retries Telegram sends on rate limits and logs errors with context.

## 8) Agent Rule Files Check

- This repository includes this `AGENTS.md` at the repo root.
- No `.cursorrules` file found.
- No `.cursor/rules/` directory found.
- No `.github/copilot-instructions.md` found.

If any of the above rule files are added later, treat them as higher-priority supplements and merge guidance into this file.

## 9) Practical Default Workflow

1. Read target crate `Cargo.toml` + touched source files.
2. Implement minimal code changes.
3. Run package-scoped checks/tests first.
4. Run workspace checks/tests for broader-impact changes.
5. Update README/docs when CLI/config behavior changes.

## 10) Quick Sanity Checklist Before Hand-off

- Builds: `cargo check --workspace`
- Tests: `cargo test --workspace` (or documented subset + rationale)
- No accidental API/config drift.
- Error messages remain actionable and user-friendly.
- New behavior has tests near changed code.
