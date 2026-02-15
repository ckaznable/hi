## 1. Root CLI Workspace Setup

- [x] 1.1 Create a new binary crate under `bin/hi` with `main.rs` as system entrypoint.
- [x] 1.2 Add the new crate to workspace members in root `Cargo.toml`.
- [x] 1.3 Add `clap` dependency and define CLI parser with `tui` subcommand.

## 2. TUI Launch Integration via Root CLI

- [x] 2.1 Extract or expose reusable TUI startup function from `package/hi-tui` so it can be invoked by root CLI.
- [x] 2.2 Wire root CLI `tui` subcommand to load config and invoke the reusable TUI startup flow.
- [x] 2.3 Keep terminal init/restore behavior identical and verify clean exit path through root CLI launch.

## 3. Streaming Chat Path in Core/TUI

- [x] 3.1 Add streaming response API support in `hi-core` chat path with provider-level fallback to non-streaming mode.
- [x] 3.2 Update TUI message handling to consume assistant stream chunks and render incremental updates.
- [x] 3.3 Implement per-response accumulation with a single reused mutable `String` buffer (`push_str`) and persist final response to history.

## 4. Validation and Regression Coverage

- [x] 4.1 Add/adjust tests for clap command parsing and `tui` dispatch behavior.
- [x] 4.2 Add/adjust tests for streaming accumulation, including single-buffer chunk accumulation and non-streaming fallback.
- [x] 4.3 Run `cargo check --workspace` and `cargo test --workspace`, then fix any regressions.
