## Summary

This change is a pure implementation-level refinement with no spec-level changes.

## ADDED Requirements

(None - this change only modifies implementation behavior, not requirements)

## MODIFIED Requirements

(None - no existing requirements are changed)

## Rationale

The Telegram adapter's long polling error handling is an implementation detail that doesn't affect the external contract or requirements of the system. The behavior from the user's perspective remains unchanged - the bot continues to poll for updates and process messages normally.

This change only affects internal logging behavior, reducing noise in stderr during normal timeout cycles.
