## Why

Users can set an LLM configuration but still fail at runtime if the endpoint, key, or model is invalid. We need a quick verification command in `main` to proactively confirm the configured LLM is reachable before normal usage.

## What Changes

- Add a `config` validation command in the main CLI flow that sends a simple `hi` request to the LLM defined in current config.
- Return clear success output when the model responds, and actionable failure output when connection/auth/model checks fail.
- Reuse existing config loading and provider selection behavior so validation matches real runtime settings.

## Capabilities

### New Capabilities
- `config-llm-validation-command`: Validate configured LLM connectivity by issuing a minimal probe request through the same runtime client path.

### Modified Capabilities
- `model-config`: Define validation expectations for configured provider/model credentials when running the new CLI validation command.

## Impact

- Affected code: main CLI command routing, config loading path, model client invocation, and user-facing CLI output.
- Affected APIs: no external API contract changes; one new CLI command/entry under existing interface.
- Dependencies/systems: existing LLM provider integrations and network calls used by current runtime model requests.
