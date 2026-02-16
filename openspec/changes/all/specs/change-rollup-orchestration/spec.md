## ADDED Requirements

### Requirement: Dependency-aware execution plan for active changes
The workflow SHALL define an implementation plan that orders active, non-archived changes by technical dependency rather than by creation date.

#### Scenario: Producing a valid execution order
- **WHEN** maintainers prepare to implement a batch of active OpenSpec changes
- **THEN** they MUST produce an ordered sequence where prerequisite changes appear before dependent changes

#### Scenario: Detecting invalid ordering
- **WHEN** a proposed sequence places a dependent change before its prerequisite
- **THEN** the workflow MUST reject that sequence and require reordering before implementation starts

### Requirement: Wave-based integration checkpoints
The workflow SHALL partition execution into waves and require verification at the end of each wave before the next wave begins.

#### Scenario: Completing a wave successfully
- **WHEN** all changes in a wave are implemented
- **THEN** package-scoped checks/tests relevant to modified crates MUST pass before advancing

#### Scenario: Blocking advancement on failed verification
- **WHEN** any required verification in a wave fails
- **THEN** the workflow MUST block progression to subsequent waves until failures are resolved

### Requirement: Final workspace validation gate
Before declaring the rollup complete, the workflow SHALL require full workspace validation to ensure cross-crate compatibility.

#### Scenario: Final completion criteria
- **WHEN** all waves are complete
- **THEN** `cargo check --workspace` and `cargo test --workspace` MUST succeed before the rollout is marked done

#### Scenario: Handling unrelated pre-existing failures
- **WHEN** workspace validation fails due to known pre-existing unrelated issues
- **THEN** the workflow MUST document those failures explicitly and distinguish them from regressions introduced by the rollup
