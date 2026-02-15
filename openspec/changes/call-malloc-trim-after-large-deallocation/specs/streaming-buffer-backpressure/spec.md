## ADDED Requirements

### Requirement: Bounded streaming queue behavior
The system MUST bound memory growth in streaming response pipelines by using bounded buffering or equivalent backpressure controls.

#### Scenario: Producer outpaces consumer
- **WHEN** streaming producers generate chunks faster than consumers can process them
- **THEN** the system MUST apply configured backpressure behavior instead of allowing unbounded queue growth

#### Scenario: Producer and consumer are balanced
- **WHEN** consumer throughput keeps pace with producer output
- **THEN** the system MUST deliver streamed chunks in order without additional loss

### Requirement: Full-queue handling policy
The system MUST define deterministic behavior when streaming buffer capacity is reached.

#### Scenario: Queue reaches capacity
- **WHEN** an enqueue operation occurs while buffer capacity is full
- **THEN** the system MUST follow the configured policy (block, drop with accounting, or coalesce) consistently for that pipeline

#### Scenario: Queue has available capacity
- **WHEN** an enqueue operation occurs while queue capacity is available
- **THEN** the system MUST enqueue the chunk and preserve output order guarantees

### Requirement: Backpressure observability
The system MUST provide runtime evidence for saturation and policy actions in streaming queues.

#### Scenario: Saturation event occurs
- **WHEN** a streaming queue enters saturation state
- **THEN** the system MUST emit structured evidence including pipeline identifier and action applied

#### Scenario: Saturation clears
- **WHEN** queue pressure returns below saturation threshold
- **THEN** the system MUST emit structured evidence that normal flow resumed
