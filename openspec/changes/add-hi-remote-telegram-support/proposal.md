## Why

The project currently provides a local TUI interface only, so it cannot receive or reply through external messaging platforms. Adding a dedicated remote communication capability now enables non-TUI usage and establishes a foundation for channel integrations, starting with Telegram.

## What Changes

- Add a new `hi-remote` capability to handle external messaging platform integration boundaries.
- Introduce Telegram as the first supported remote channel, including inbound message intake and outbound reply delivery.
- Define how remote messages map to `hi-core` chat session flow so remote users can interact with the same agent pipeline.
- Establish configuration and runtime expectations for running a remote adapter process alongside existing CLI/TUI workflows.

## Capabilities

### New Capabilities
- `hi-remote`: Provide a remote integration layer for external messaging platforms, with initial Telegram support.

### Modified Capabilities
- None.

## Impact

- Affected code: new workspace package under `package/` for remote integration logic, plus integration points with `hi-core` session APIs and shared configuration.
- Affected systems: runtime now includes an external messaging adapter path in addition to the local TUI path.
- External dependencies/API: adds Telegram Bot API communication as a supported external interface.
