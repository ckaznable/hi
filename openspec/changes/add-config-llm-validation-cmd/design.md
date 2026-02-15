## Context

`hi` currently validates configuration mostly at load/parse time, but users still discover endpoint/auth/model issues only when they start a real workflow (for example `tui`/`remote`). This change introduces a dedicated command path to verify the configured runtime model connection by sending a minimal `hi` probe through the same provider/client stack used in production flows.

## Goals / Non-Goals

**Goals:**
- Add a CLI command under the main command surface to validate current config against the configured LLM.
- Execute a minimal probe request (`hi`) through existing model invocation path to detect endpoint/auth/model problems early.
- Keep provider behavior consistent with current config resolution and multi-provider support.
- Return clear CLI output for success and actionable failure categories.

**Non-Goals:**
- No new provider integrations in this change.
- No schema redesign for `config.json`.
- No interactive fix-up flow or auto-repair of invalid configs.

## Decisions

### 1) Add validation as a dedicated CLI command in main routing

- Decision: introduce a `config` validation command path from `main` command parsing/dispatch.
- Rationale: keeps validation discoverable and explicit, instead of coupling it to side effects in `tui`/`remote` startup.
- Alternative considered: run connectivity checks automatically on every startup. Rejected because it adds mandatory network latency and can break offline workflows.

### 2) Reuse existing config load and runtime client creation path

- Decision: validation command uses the same config loading and provider selection logic already used by runtime flows.
- Rationale: prevents environment drift where validation passes but runtime fails due to different initialization logic.
- Alternative considered: build a separate lightweight HTTP probe layer per provider. Rejected because it duplicates provider-specific behavior and increases maintenance risk.

### 3) Use a minimal chat completion probe (`hi`) with strict output mapping

- Decision: send a minimal request payload with user content `hi` to the configured model and treat any normal model response as pass.
- Rationale: validates full request path (endpoint, auth, model routing, request format) with minimal token and time cost.
- Alternative considered: provider-specific health endpoints. Rejected because not all providers expose equivalent health APIs and these may bypass model-level checks.

### 4) Standardize error classification for operator-friendly diagnostics

- Decision: map failures into high-level categories (config invalid, auth failed, endpoint unreachable/timeout, model not found/unsupported, unknown provider error) and print concise remediation hints.
- Rationale: improves debugging quality over raw transport/provider error output.
- Alternative considered: print raw error only. Rejected because error messages vary by provider and are often noisy for end users.

## Risks / Trade-offs

- [Probe may consume provider quota/tokens] -> Mitigation: keep payload minimal (`hi`), single request, no retry storm by default.
- [Network flakiness can produce false negatives] -> Mitigation: classify transient transport errors clearly and avoid mutating config state.
- [Provider-specific error text inconsistency] -> Mitigation: normalize to stable high-level categories while preserving original detail in debug output path.
- [Command naming ambiguity (`config` vs future subcommands)] -> Mitigation: keep command semantics narrow now and allow later expansion under the same namespace.

## Migration Plan

1. Add CLI command parsing and dispatch path for config validation.
2. Implement validation runner that loads config and sends one `hi` probe using existing model client construction.
3. Add error mapping/formatting for actionable CLI outcomes.
4. Add tests for parse/dispatch and success/failure behavior across representative error classes.
5. Update CLI/help documentation to describe command purpose and expected output.

Rollback strategy: remove command wiring and validation runner; existing `tui`/`remote` startup behavior remains unchanged because this feature is additive.

## Open Questions

- Should this command validate only the primary model, or also optionally validate `small_model` when configured?
- Should a future non-network mode (schema-only local validation) be included as a separate flag, or remain a separate command?
