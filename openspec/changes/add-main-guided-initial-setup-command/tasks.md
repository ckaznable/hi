# add-main-guided-initial-setup-command — Tasks

## Status: COMPLETE

## Summary

Enhanced `hi init` to walk the user through provider/model/API key/context window selection interactively. The old static-template behavior is preserved via `hi init --quick`.

## Tasks

- [x] **shared/src/config.rs** — Added `guided_init_config()`: prompts user for provider (1–5), model (with sensible defaults per provider), API base (openai-compatible only), API key (skipped for Ollama), context window (with defaults)
- [x] **shared/src/config.rs** — Added `prompt_with_default()` and `prompt_required()` helper functions using `BufRead`/`Write` traits for testability
- [x] **shared/src/config.rs** — 3 new unit tests for prompt helpers
- [x] **src/main.rs** — Added `--quick` flag to `InitCommand` struct
- [x] **src/main.rs** — `hi init` now calls `guided_init_config()` by default; `hi init --quick` calls `init_config()` (old template behavior)
- [x] **src/main.rs** — Added `test_parse_init_quick_flag` test; updated existing `test_parse_init_command`
- [x] **README.md** — Updated both Quick Start sections and CLI Subcommands to reflect guided setup

## Verification

- `cargo check --workspace` — clean
- `cargo test --workspace` — 204 tests, 0 failures
