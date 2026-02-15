## Context

The current provider model supports `openai`, `anthropic`, `gemini`, and `ollama`. Configuration validation currently requires `api_key` for all providers except `ollama`, and agent construction has provider-specific client builders in `hi-core/src/provider.rs`. The requested change introduces explicit support for OpenAI-compatible endpoints so users can target gateways/proxies that speak OpenAI chat-completions protocol but are not the official OpenAI endpoint.

This is cross-cutting because provider semantics affect config parsing/validation, main model creation, and secondary model resolution used by heartbeat/scheduler via `SmallModelConfig`.

## Goals / Non-Goals

**Goals:**
- Add a first-class provider mode for OpenAI-compatible endpoints.
- Keep existing OpenAI provider behavior stable for current users.
- Support OpenAI-compatible provider in both primary `ModelConfig` and `small_model` / inline `ModelRef`.
- Define deterministic auth behavior for compatible endpoints where API key may be optional.

**Non-Goals:**
- Supporting non-chat-completions APIs.
- Adding endpoint-specific feature negotiation or capability probing.
- Changing tool execution model, session architecture, or TUI behavior.
- Introducing new network dependencies or protocol adapters beyond existing `rig` OpenAI client path.

## Decisions

### 1) Add new provider enum value
- Decision: Add `Provider::OpenAICompatible` serialized as `openai-compatible`.
- Rationale: Distinguishes official OpenAI mode from generic compatible endpoints without overloading existing `openai` semantics.
- Alternative considered: Reuse `openai` and infer compatible mode from `api_base` presence.
  - Rejected because it hides intent and makes validation/auth semantics ambiguous.

### 2) Reuse OpenAI client path for compatible provider
- Decision: Route `openai-compatible` through the same `rig::providers::openai::CompletionsClient` builder, with configurable `base_url` and model.
- Rationale: Protocol compatibility means we can reuse existing, tested request path and avoid duplicate client implementations.
- Alternative considered: Add a new custom HTTP client flow.
  - Rejected due to complexity and divergence risk from existing provider architecture.

### 3) Provider-specific auth validation rules
- Decision: Keep API key required for `openai`; allow optional API key for `openai-compatible` and `ollama`.
- Rationale: Many compatible gateways support keyless local/network auth; this change should not force fake keys.
- Alternative considered: Require API key for all OpenAI-protocol providers.
  - Rejected because it blocks common local proxy deployments.

### 4) Keep multi-model behavior unchanged except provider support
- Decision: `small_model` and inline `ModelRef::Inline` continue using existing resolution logic, but now may carry `openai-compatible` provider and its validation semantics.
- Rationale: Preserves current architecture while extending supported provider values.
- Alternative considered: Introduce separate compatible model type.
  - Rejected as unnecessary duplication.

### 5) Backward compatibility and naming
- Decision: Existing config values remain valid; new behavior is additive.
- Rationale: Avoids migration burden and preserves currently passing tests/configs.

## Risks / Trade-offs

- [Risk] Some “OpenAI-compatible” services deviate subtly from official API behavior -> Mitigation: document that compatibility assumes chat-completions parity; surface provider errors directly.
- [Risk] Optional API key for compatible mode may hide misconfiguration in environments that still require keys -> Mitigation: clear runtime error propagation from endpoint responses; include validation tests for optional-but-present behavior.
- [Risk] Enum expansion can affect serde expectations in existing tooling -> Mitigation: additive enum value only; keep existing serialized names untouched.

## Migration Plan

1. Add provider enum value and validation updates in `shared`.
2. Extend provider factory in `hi-core` to branch on `openai-compatible` using OpenAI client path.
3. Add tests for config parsing/validation and multi-model resolution with compatible provider.
4. Rollout with backward-compatible defaults (no existing config changes required).

Rollback: remove/disable `openai-compatible` enum branch and validation allowances; existing providers continue unaffected.

## Open Questions

- Should `openai-compatible` default `api_base` be required explicitly, or should it fallback to OpenAI default when omitted?
- Do we want to enforce URL shape checks for `api_base` at config-validate time, or leave URL validation to client construction/runtime?
