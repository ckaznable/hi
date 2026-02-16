# add-skills-slash-command-and-prompt-prepend — Tasks

## Status: COMPLETE

## Summary

Added `/skills` slash command to both TUI and Telegram, and skill body content is prepended to the system preamble via `build_preamble()`.

## Tasks

- [x] **hi-core/src/skills.rs** — `Skill` struct, `load_skills()`, `parse_frontmatter()`, `build_preamble()` (pre-existing)
- [x] **hi-core/src/session.rs** — Added `pub fn skills()` accessor returning `&[Skill]`
- [x] **hi-core/src/session.rs** — Skill bodies merged into preamble via `build_preamble()` in `ChatSession::new()` (pre-existing)
- [x] **hi-tui/src/lib.rs** — `/skills` command: captures skill list before async move, displays as system message
- [x] **hi-remote/src/telegram.rs** — `/skills` command: calls `format_skills()` helper
- [x] **hi-remote/src/telegram.rs** — `format_skills()` function: loads skills from config dir via `hi_core::skills::load_skills()`
- [x] **hi-remote/src/telegram.rs** — Added `/skills` to `/help` output

## Verification

- `cargo check --workspace` — clean
- `cargo test --workspace` — 200 tests, 0 failures
