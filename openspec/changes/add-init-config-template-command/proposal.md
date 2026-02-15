## Why

Users currently need to create `config.json` manually before running the app, which slows first-time setup and causes avoidable formatting mistakes. Adding an `init` command provides a consistent, guided bootstrap path and reduces setup friction.

## What Changes

- Add a new CLI subcommand `init` to generate a starter configuration template file.
- Create the template at the standard config location (`config_dir()/config.json`) with valid baseline fields.
- Return clear CLI output for success, missing parent directory creation, and existing-file handling.
- Document how to use `hi init` in user-facing docs.

## Capabilities

### New Capabilities
- `config-init-command`: Provide a CLI command that writes a valid config template so users can start quickly without manual file creation.

### Modified Capabilities
- None.

## Impact

- **CLI entrypoint**: `bin/hi/src/main.rs` adds `init` command wiring and command dispatch.
- **Config path handling**: shared path/config utilities are used to resolve and create target paths safely.
- **User docs**: README and command help text are updated to include initialization workflow.
