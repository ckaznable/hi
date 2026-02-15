## Context

The `hi` root CLI currently parses only subcommands (`tui`, `remote`) and does not expose any shared option for configuration location. Both runtime entrypoints (`hi_tui::run_tui()` and `hi_remote::run_remote()`) call `ModelConfig::load()`, which always reads `config.json` from `ProjectDirs::config_dir("hi")`. This fixed default path works for normal local usage but creates friction for testing, multi-environment setups, and running with alternate config files.

## Goals / Non-Goals

**Goals:**
- Add a shared top-level CLI parameter to specify config file path.
- Preserve existing default behavior when the parameter is absent.
- Ensure both `tui` and `remote` commands use the same resolved config path behavior.
- Keep changes minimal and backward-compatible with existing config schema.

**Non-Goals:**
- Changing config JSON structure or validation semantics beyond path resolution.
- Introducing new config discovery mechanisms beyond CLI override + existing default.
- Refactoring unrelated startup/session logic.

## Decisions

1. Add a global CLI option on `Cli` (root command), not per-subcommand.
   - Decision: Define an optional path argument (e.g. `--config` / `-c`) at top level so it is shared by all subcommands.
   - Rationale: Matches the requirement for a common parameter and avoids duplicated subcommand options.
   - Alternative considered: Add separate `--config` to each subcommand. Rejected due to duplication and inconsistent UX risk.

2. Introduce explicit-path loading API while preserving current default loader.
   - Decision: Keep `ModelConfig::load()` for default path behavior and add a path-aware variant (e.g. `ModelConfig::load_from_path(...)`).
   - Rationale: Minimizes breakage and keeps existing call sites valid while enabling explicit override.
   - Alternative considered: Change `ModelConfig::load()` signature to require optional path. Rejected due to wider ripple and less clear API intent.

3. Resolve config path at command boundary and pass resolved intent downward.
   - Decision: Parse optional config path in root CLI, then pass it into `run_tui` / `run_remote` so both commands share identical precedence rules.
   - Rationale: Centralized resolution policy and fewer divergence points.
   - Alternative considered: Let each package independently resolve CLI/env/default path. Rejected due to duplication and drift risk.

4. Preserve missing-file behavior semantics with contextual error path.
   - Decision: Keep failure on missing config file, but error should reference the actual attempted path (explicit or default).
   - Rationale: Maintains existing reliability expectation and improves diagnosability.
   - Alternative considered: Silent fallback when explicit path is invalid. Rejected because user intent should fail fast.

## Risks / Trade-offs

- [Signature ripple across crates] -> Mitigation: keep interfaces narrow (`Option<PathBuf>` at boundary) and avoid unrelated refactors.
- [Inconsistent behavior between commands] -> Mitigation: shared root parsing and shared loading path logic.
- [Ambiguity if relative paths are used] -> Mitigation: document that relative paths are resolved from current working directory and surface resolved path in errors.
- [Test coverage gaps for new flag] -> Mitigation: add parser tests for global option and unit tests for default vs explicit load paths.

## Migration Plan

1. Add root CLI optional config path argument and parse tests.
2. Add config loading API for explicit path while preserving default loader.
3. Update `run_tui` and `run_remote` signatures to accept optional config path.
4. Thread parsed argument from main to command handlers.
5. Add/adjust tests for parser and config path resolution behavior.

Rollback strategy: remove the new root argument wiring and keep `ModelConfig::load()` default-only flow; no data migration is involved.

## Open Questions

- Should an environment variable fallback (e.g. `HI_CONFIG`) be included now, or deferred to a separate change?
- Should standalone binary entrypoints mirror this option immediately, or only root `hi` CLI in this change?
