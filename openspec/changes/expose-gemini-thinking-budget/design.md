## Context

The rig library provides a `ThinkingConfig` option to enable extended thinking capabilities in models that support it (e.g., Gemini 2.0 with thinking mode). Currently, users cannot configure this through the `config.json` file.

## Goals / Non-Goals

**Goals:**
- Expose rig's `ThinkingConfig` through the application's config
- Support both `type` (enabled/auto) and `budget_tokens` options

**Non-Goals:**
- Provider-specific thinking configuration beyond rig's generic options
- Runtime thinking configuration changes (config-only for now)

## Decisions

### Decision 1: Config structure

**Choice:** Add `thinking` as top-level field in `ModelConfig`

```json
{
  "thinking": {
    "type": "enabled",
    "budget_tokens": 1024
  }
}
```

**Rationale:** Simple, consistent with other optional config fields like `heartbeat`, `compact`.

### Decision 2: Integration point

**Choice:** Pass `ThinkingConfig` to `create_agent_from_parts()` and similar constructors

**Rationale:** Keeps provider-specific logic in `provider.rs` where agent creation happens.

## Risks / Trade-offs

- **[Risk] Provider compatibility** → Not all providers support thinking. Rig handles this gracefully by ignoring unsupported options.
- **[Risk] Token budget validation** → Need to ensure budget is reasonable (e.g., positive integer).
