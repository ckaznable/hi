## Why

The current system only supports a single model configuration, and all features share the same LLM. We need to add a heartbeat system (periodic background tasks) and scheduled jobs. These features do not require the primary model and can use a lighter `small_model` to reduce cost. At the same time, runtime behavior should prioritize minimal memory usage.

## What Changes

- Add a new `small_model` field in config to specify a lightweight model for simple tasks
- Add optional `model` field to heartbeat config to choose the heartbeat model (default: `small_model` or primary model)
- Add optional `model` field to scheduler config so each scheduled task can choose its own model
- Add multi-model management: create clients on demand and share client instances for the same provider
- Minimize memory usage: lazy-init model clients and release unused resources promptly

## Capabilities

### New Capabilities

- `heartbeat`: periodic background tasks (for example health checks or summary generation), with configurable model
- `scheduler`: cron-like scheduled jobs with per-task model selection
- `multi-model`: multi-model management across primary model + small_model + feature-specific inline models, with lazy init and shared clients

### Modified Capabilities

- `model-config`: add `small_model` plus heartbeat/scheduler config sections

## Impact

- **shared**: extend `ModelConfig` and introduce `ModelRef`
- **hi-core**: add heartbeat module, scheduler module, and multi-model pool
- **memory**: client lazy init + sharing to reduce unnecessary allocation
- **dependencies**: may add `cron` or `tokio-cron-scheduler` for scheduling
