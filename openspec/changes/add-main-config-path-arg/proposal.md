## Why

The CLI currently always reads configuration from the default `ProjectDirs::config_dir("hi")/config.json` location, which makes non-standard deployment and local testing workflows inconvenient. We need a shared top-level parameter so users can point to a specific config file while preserving current default behavior.

## What Changes

- Add a shared root CLI option to specify a config file path for `hi` commands (for example `tui` and `remote`).
- Update config loading behavior to prefer the explicit CLI path when provided.
- Keep existing default config path behavior when the shared parameter is not provided.
- Define validation and error behavior for invalid or missing user-specified config paths.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `model-config`: Extend config discovery requirements to support an optional CLI-provided config path override, with fallback to the existing default path.

## Impact

- Affected code: root CLI argument parsing in `bin/hi/src/main.rs`, shared config loading in `package/shared/src/config.rs`, and startup flows that currently call `ModelConfig::load()`.
- Affected behavior: command startup config resolution path and related error messages.
- Backward compatibility: existing users without the new parameter continue using the current default config location.
