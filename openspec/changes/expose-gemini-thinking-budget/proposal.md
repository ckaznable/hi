## Why

The rig library provides a `ThinkingConfig` option to enable and configure extended thinking (e.g., Gemini's thinking mode). Currently, there's no way to configure this from the user's `config.json`, limiting the ability to use advanced model features.

## What Changes

- Add `thinking` field to `ModelConfig` in `shared/src/config.rs`
- The field accepts a `ThinkingConfig` object with:
  - `type`: thinking mode type (e.g., "enabled", "auto")
  - `budget_tokens`: thinking budget in tokens
- Pass `ThinkingConfig` to rig's agent creation

## Capabilities

### New Capabilities
- `thinking-config`: Expose rig's ThinkingConfig to user-configurable options

### Modified Capabilities
- `model-config`: Add optional `thinking` field to model configuration

## Impact

- **Affected code**:
  - `package/shared/src/config.rs` — add `ThinkingConfig` struct and field
  - `package/hi-core/src/provider.rs` — pass thinking config to rig agent creation
- **Config**: New `thinking` field in `config.json`
- **Dependencies**: No new dependencies (rig already has ThinkingConfig)
