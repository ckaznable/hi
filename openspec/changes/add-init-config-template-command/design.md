## Context

The project currently expects users to provide `config.json` before running `hi tui` or `hi remote`. This creates onboarding friction and avoidable formatting errors. We need a built-in CLI path to create a valid starter config at the standard config location while preserving existing files safely.

## Goals / Non-Goals

**Goals:**
- Add a new `hi init` subcommand in the root CLI.
- Generate a valid starter `config.json` at `config_dir()/config.json`.
- Ensure behavior is predictable when parent directories are missing or config already exists.
- Keep implementation aligned with existing `shared` path/config primitives.

**Non-Goals:**
- No interactive setup wizard in this change.
- No provider-specific profile generator (single baseline template only).
- No automatic merge/migration of existing user config.

## Decisions

### 1. CLI surface and dispatch

We add `Init` as a new subcommand in `bin/hi/src/main.rs`, alongside `Tui` and `Remote`.

Rationale:
- Matches existing command structure and keeps discoverability high (`hi --help`).
- Avoids introducing a nested command hierarchy for a single bootstrap action.

Alternative considered:
- `hi config init` nested command. Rejected for now because current CLI is flat and only has top-level modes.

### 2. Target path and file creation behavior

`hi init` resolves the path using existing shared path helpers and targets `config_dir()/config.json`. If the directory does not exist, it is created before writing.

Behavior:
- If `config.json` does not exist: create it with template content.
- If `config.json` already exists: return an error and do not overwrite.

Rationale:
- Uses established project path semantics across platforms.
- Prevents accidental loss of user configuration.

Alternative considered:
- Always overwrite existing file. Rejected because it is unsafe and surprising.

### 3. Template content contract

The generated template is valid JSON and includes minimal fields required by current config validation.

Baseline template:

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxxx",
  "context_window": 128000
}
```

Rationale:
- This shape is already documented in README and accepted by `ModelConfig` parsing.
- Keeps the first-run file short and immediately editable.

Alternative considered:
- Generate a larger template including optional blocks (`small_model`, `heartbeat`, `schedules`, `compact`, `remote`). Rejected to keep onboarding simple.

### 4. User feedback and failure semantics

`hi init` returns explicit CLI messages for:
- Successful template creation (including resolved path)
- Existing file (no overwrite)
- Filesystem write/path errors (permission, invalid path, etc.)

Rationale:
- First-run command quality is mostly UX; clear status text reduces support/debug effort.

### 5. Documentation alignment

README command sections will include `hi init` in install/startup flow so users discover initialization before trying TUI/remote.

Rationale:
- Keeps docs and CLI behavior synchronized.

## Risks / Trade-offs

- Existing-file refusal may require manual delete for users who want to regenerate quickly -> Mitigation: explicit error message that explains file already exists and path.
- Minimal template may not match all provider choices out of the box -> Mitigation: keep template intentionally small and point users to README provider examples.
- Config path creation can fail on restricted environments -> Mitigation: propagate actionable filesystem errors with path context.
