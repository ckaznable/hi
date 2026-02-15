## Why

The root CLI currently depends on `clap` and always links TUI support, which increases default build surface and prevents lightweight installs that only need non-TUI commands. We need to migrate to `argh` and make TUI opt-in so default builds stay minimal while preserving a clear path to enable TUI.

## What Changes

- Replace root CLI argument parsing in `bin/hi` from `clap` derives/APIs to `argh` equivalents for existing command routes (`init`, `remote`, `config validate`, and `tui`).
- Make `hi-tui` an optional dependency in `hi-cli`, controlled by a dedicated Cargo feature, with default feature set excluding TUI.
- Define user-facing behavior for `tui` command when TUI feature is disabled (compile-time gated command surface or explicit disabled-path messaging).
- Update command/help documentation to reflect feature-gated TUI availability and enablement instructions.

## Capabilities

### New Capabilities
- `root-cli-parser-and-feature-gating`: Define root CLI parsing behavior with `argh` and feature-gated TUI command availability.

### Modified Capabilities
- `chat-tui`: Clarify runtime entry expectations when TUI is built as an optional feature rather than always present in default `hi-cli` builds.

## Impact

- Affected code: `bin/hi/src/main.rs`, `bin/hi/Cargo.toml`, workspace dependency definitions in `Cargo.toml`, and CLI docs in `README.md`.
- Dependency impact: remove `clap` usage from root CLI and introduce `argh`; make `hi-tui` optional for `hi-cli` via feature wiring.
- Build/runtime impact: default `hi-cli` build excludes TUI path; TUI remains available when explicitly enabled.
