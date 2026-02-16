## ADDED Requirements

### Requirement: LLM tool can edit heartbeat content
The system SHALL provide an LLM-invokable tool that updates heartbeat content in managed storage.

#### Scenario: Replace heartbeat content successfully
- **WHEN** the tool is called with a valid full replacement payload for heartbeat content
- **THEN** the system writes the updated content and returns a success result

### Requirement: Heartbeat edit operation validates target and payload
The system SHALL validate heartbeat edit requests before writing changes.

#### Scenario: Reject empty content update
- **WHEN** the edit tool is called with missing or empty required content
- **THEN** the system returns a validation error and SHALL NOT persist any change

#### Scenario: Reject unsupported edit mode
- **WHEN** the edit tool is called with an unsupported edit mode or malformed edit payload
- **THEN** the system returns an error describing invalid input and SHALL NOT persist any change

### Requirement: Heartbeat edits preserve managed-file constraints
The system SHALL only mutate the heartbeat file in the managed runtime data location and SHALL NOT allow arbitrary path writes.

#### Scenario: Edit request attempts external path mutation
- **WHEN** a heartbeat edit request includes path-like input targeting a non-managed location
- **THEN** the system rejects the request and writes no changes outside managed storage

### Requirement: Heartbeat mutation outcomes are machine-actionable
The system SHALL return structured success and error results for heartbeat edit operations.

#### Scenario: Edit conflict or write failure returns actionable error
- **WHEN** a heartbeat edit fails due to write conflict or storage failure
- **THEN** the system returns an error result that includes failure reason suitable for automated retry or correction
