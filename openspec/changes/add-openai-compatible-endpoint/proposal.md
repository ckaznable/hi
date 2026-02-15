## Why

Current provider options are fixed and optimized for official vendor endpoints. Users who run OpenAI-compatible gateways (self-hosted proxy, model router, or local API) cannot configure them as a first-class provider with predictable auth and endpoint behavior. Adding explicit OpenAI-compatible endpoint support now unblocks broader deployment targets without forcing provider-specific workarounds.

## What Changes

- Add a new provider mode for OpenAI-compatible APIs, including configurable base URL and model name routing.
- Define config behavior for authentication in OpenAI-compatible mode (allow optional/empty API key when endpoint permits it).
- Ensure agent creation and request flow reuse OpenAI chat-completions format against custom endpoints.
- Extend model selection flow so `small_model`, heartbeat, and schedules can target OpenAI-compatible endpoints consistently.

## Capabilities

### New Capabilities
- `openai-compatible-provider`: Support OpenAI-compatible chat-completions endpoints as a dedicated provider option with configurable base URL and auth behavior.

### Modified Capabilities
- `model-config`: Extend provider/config validation rules to include OpenAI-compatible provider semantics.
- `multi-model`: Allow secondary model references (`small_model` and inline model refs) to use OpenAI-compatible endpoints.

## Impact

- Affected specs: `model-config`, `multi-model`, and new `openai-compatible-provider`.
- Affected code: provider enum/config parsing, validation logic, agent factory construction, and model resolution paths used by heartbeat/scheduler.
- External API behavior: outbound requests may target non-OpenAI hosts that implement OpenAI-compatible interfaces.
