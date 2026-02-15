## Context

`hi-cli` currently parses commands with `clap` in `bin/hi/src/main.rs` and always depends on `hi-tui` via `bin/hi/Cargo.toml`. This couples default builds to TUI dependencies even when users only need `init`, `remote`, or `config validate`. The change spans parser migration (`clap` -> `argh`), Cargo feature wiring, command-dispatch behavior, and user documentation.

Constraints:
- Preserve existing command intents (`init`, `remote`, `config validate`, `tui`) and exit/error patterns.
- Keep root CLI behavior deterministic for builds both with and without TUI.
- Avoid broad architectural refactors outside CLI parsing and feature gating.

## Goals / Non-Goals

**Goals:**
- Replace root CLI parser implementation from `clap` to `argh` without changing command semantics.
- Make TUI support opt-in in `hi-cli` by defaulting features to exclude `hi-tui`.
- Define explicit runtime behavior when users invoke `tui` in non-TUI builds.
- Keep docs/build instructions aligned with feature-gated behavior.

**Non-Goals:**
- Rewriting `hi-tui` internals or chat interaction behavior.
- Changing `hi-remote` or model validation logic beyond command parser adaptation.
- Introducing new commands unrelated to parser migration or TUI feature gating.

## Decisions

1. Migrate parser derives and command types to `argh` in `bin/hi/src/main.rs`.
   - Rationale: aligns with change goal and reduces parser dependency weight.
   - Alternative considered: keep `clap` and only gate TUI. Rejected because it does not satisfy requested parser migration.

2. Make `hi-tui` optional in `bin/hi/Cargo.toml` and add a dedicated `tui` Cargo feature with `default = []`.
   - Rationale: default builds avoid linking TUI dependencies and remain minimal.
   - Alternative considered: workspace-level feature only. Rejected because `hi-cli` owns command dispatch and should directly control optional dependency linkage.

3. Keep `tui` command in CLI surface and provide a clear disabled-path response when the feature is not enabled.
   - Rationale: preserves a stable command name while giving actionable feedback instead of silent removal ambiguity.
   - Alternative considered: compile out the `tui` subcommand entirely. Rejected due to discoverability and behavior drift across builds.

4. Update `README.md` command docs to show feature-enabled invocation paths.
   - Rationale: without docs alignment, users will see command mismatch in default builds.
   - Alternative considered: no docs change. Rejected because behavior change is user-facing.

## Risks / Trade-offs

- [Parser behavior differences between `clap` and `argh`] -> Add command parsing tests for valid/invalid command forms and preserve current failure expectations where possible.
- [User confusion when `tui` is unavailable in default build] -> Return explicit guidance in error output (how to rebuild/run with `--features tui`) and update README examples.
- [Feature-gating mistakes causing compile failures in one build mode] -> Validate both `cargo check -p hi-cli` and `cargo check -p hi-cli --features tui` during implementation.
- [Help text/output drift from `clap` to `argh`] -> Document expected output differences and keep command list accurate in README.

## Migration Plan

1. Update dependency manifests:
   - Replace/remove `clap` usage for `hi-cli` and introduce `argh`.
   - Set `hi-tui` as optional in `bin/hi/Cargo.toml` and define `tui` feature (`default = []`).
2. Refactor `bin/hi/src/main.rs` parser types/dispatch from `clap` APIs to `argh` equivalents.
3. Add feature-aware dispatch branch for `tui` when feature disabled.
4. Update CLI parser tests for `argh` parsing entry points and feature-aware behavior.
5. Update README command examples for default vs `--features tui` usage.
6. Validate both default and TUI-enabled builds/tests.

Rollback strategy:
- Revert parser and manifest changes together, restoring `clap` and unconditional `hi-tui` dependency if compatibility issues appear.

## Open Questions

- Should disabled `tui` path return a standard non-zero exit with concise one-line hint, or include a fuller multi-line remediation message?
- Should installation docs prefer `cargo install --path bin/hi --features tui` as primary path, or keep default install first and mention TUI as optional add-on?
