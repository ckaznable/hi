## 1. CLI Command Surface

- [x] 1.1 Add a config validation command entry in `bin/hi/src/main.rs` command parsing and dispatch flow.
- [x] 1.2 Wire the new command to reuse existing config loading behavior before running validation.
- [x] 1.3 Add/adjust CLI parser tests to cover command parsing and command routing.

## 2. Validation Probe Execution

- [x] 2.1 Implement a validation runner that sends a single `hi` probe to the primary configured model using the existing provider client path.
- [x] 2.2 Ensure probe request construction follows the same runtime provider selection and client initialization code path used by normal chat flows.
- [x] 2.3 Guarantee validation is read-only and does not mutate `config.json` or other persisted state.

## 3. Outcome Classification and User Feedback

- [x] 3.1 Add error mapping for validation outcomes: auth failure, endpoint/network failure, model-not-available, config incompatibility, and unknown provider/runtime errors.
- [x] 3.2 Print concise success output when probe returns a valid model response.
- [x] 3.3 Print actionable failure output with remediation hints tied to each error class.

## 4. Verification and Documentation

- [x] 4.1 Add unit tests for successful probe flow and representative failure classifications.
- [x] 4.2 Add tests for default config validation path and invalid provider-config compatibility handling.
- [x] 4.3 Update CLI/help documentation to include the new config validation command and expected output behavior.
