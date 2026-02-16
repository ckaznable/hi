## 1. Assess Current State

- [x] 1.1 Audit remaining println!/eprintln! across the workspace
- [x] 1.2 Classify each call site as user-facing CLI output vs internal logging

## 2. Convert Internal Logging to Tracing

- [x] 2.1 Confirm all non-user-facing eprintln! already converted to tracing macros (done by add-log-crate)
- [x] 2.2 Keep user-facing println! in src/main.rs (init success, config validate output)
- [x] 2.3 Keep user-facing eprintln! in src/main.rs (TUI feature gate, config validate errors)
- [x] 2.4 Keep pre-init fallback eprintln! in shared/src/logging.rs (fires before tracing is initialized)

## 3. Verification

- [x] 3.1 cargo check --workspace passes
- [x] 3.2 cargo test --workspace passes
- [x] 3.3 No non-user-facing println!/eprintln! remains in production code

Note: The original tasks referenced log+env_logger, but the codebase uses tracing+tracing-subscriber (added by add-log-crate). This change is superseded — all actionable conversions were completed by add-log-crate.
