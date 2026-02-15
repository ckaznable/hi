## Why

The workspace currently launches the TUI by running the `hi-tui` crate directly, which does not provide a single top-level executable entrypoint for the system. We need a dedicated root CLI binary so future commands can be added behind one consistent command surface.

## What Changes

- Add a new workspace member under `bin/` containing the system-level `main` entrypoint.
- Introduce a CLI interface using `clap` in the new entrypoint binary.
- Add an initial `tui` subcommand that starts the existing `hi-tui` application flow.
- Wire workspace configuration so the new binary crate builds and runs with the rest of the workspace.

## Capabilities

### New Capabilities
- `root-cli-entrypoint`: Provide a single root executable for the `hi` system with subcommand routing.

### Modified Capabilities
- `chat-tui`: Update startup requirements so TUI can be launched through the new root CLI `tui` command path.

## Impact

- Affected code: workspace root `Cargo.toml`, new crate under `bin/`, and TUI launch integration points.
- Dependencies: add `clap` to the new binary crate.
- Runtime/API impact: introduces a canonical command-line entrypoint while preserving existing TUI behavior.
