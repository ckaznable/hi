## Why

Telegram long polling with `getUpdates` uses a timeout parameter to wait for updates. When the timeout is reached, Telegram's API returns an empty response - this is normal operation, not an error. However, the current implementation treats all errors from `get_updates()` equally and prints them to stderr, causing noisy logging during normal timeout cycles.

## What Changes

- Modify error handling in `run_polling_loop()` to distinguish between timeout/network errors and actual failures
- Silently ignore timeout errors (no log output)
- Continue polling normally after timeout
- Only log meaningful errors (API errors, authentication failures, etc.)

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. Replace <name> with kebab-case identifier -->
- (none - this is a refinement of existing behavior)

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing -->
- (none - implementation detail only, no behavior change to users)

## Impact

- Affected code: `package/hi-remote/src/telegram.rs`
- No API or behavioral changes to end users
- Reduced stderr noise during normal polling operation
