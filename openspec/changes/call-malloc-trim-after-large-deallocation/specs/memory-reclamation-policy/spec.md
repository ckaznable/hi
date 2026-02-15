## ADDED Requirements

### Requirement: Large deallocation reclamation policy
The system MUST define and apply a deterministic policy for large deallocation events in long-lived chat flows.

#### Scenario: Large history release path is detected
- **WHEN** a history compaction or reset operation releases memory above the configured large-release threshold
- **THEN** the system MUST execute the configured reclamation policy for that release path

#### Scenario: Small release path is detected
- **WHEN** a compaction or reset operation releases memory below the configured threshold
- **THEN** the system MUST skip expensive reclamation actions and continue normal flow

### Requirement: Platform-safe allocator trimming
The system MUST treat allocator trimming as platform-conditional behavior and MUST remain correct when trimming is unavailable.

#### Scenario: Supported allocator/platform
- **WHEN** runtime executes on a supported allocator/platform combination for trim operations
- **THEN** the system MUST allow `malloc_trim(0)` (or equivalent) as a best-effort optimization after a qualifying large release

#### Scenario: Unsupported allocator/platform
- **WHEN** trim capability is unavailable for the runtime platform
- **THEN** the system MUST perform no trim call and MUST preserve functional behavior

### Requirement: Reclamation observability
The system MUST expose enough runtime evidence to verify whether large-release reclamation policy was evaluated and applied.

#### Scenario: Qualifying release occurs
- **WHEN** a qualifying large-release event is processed
- **THEN** the system MUST emit structured evidence indicating threshold evaluation and policy action taken

#### Scenario: Non-qualifying release occurs
- **WHEN** a release event does not qualify
- **THEN** the system MUST emit structured evidence that policy evaluation occurred and trim was skipped
