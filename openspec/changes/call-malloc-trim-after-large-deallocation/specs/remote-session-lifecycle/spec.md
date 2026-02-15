## ADDED Requirements

### Requirement: Remote session lifecycle bounds
The system MUST enforce lifecycle bounds for Telegram remote chat sessions to prevent unbounded retention of inactive session memory.

#### Scenario: Idle session exceeds TTL
- **WHEN** a chat session remains inactive longer than the configured session TTL
- **THEN** the system MUST evict that session from in-memory ownership

#### Scenario: Session remains active
- **WHEN** a chat continues exchanging messages within TTL
- **THEN** the system MUST retain and reuse the same session for continuity

### Requirement: Capacity-based eviction safety
The system MUST support a maximum in-memory remote session capacity and apply deterministic eviction when capacity is exceeded.

#### Scenario: Capacity exceeded by new chat
- **WHEN** a new chat session is created and total session count would exceed configured capacity
- **THEN** the system MUST evict sessions according to the configured policy before admitting the new session

#### Scenario: Capacity not exceeded
- **WHEN** session count remains within configured capacity
- **THEN** the system MUST not evict active sessions for capacity reasons

### Requirement: Session lifecycle observability
The system MUST expose lifecycle events so operators can verify retention, reuse, and eviction behavior.

#### Scenario: Eviction occurs
- **WHEN** a session is evicted due to TTL or capacity rules
- **THEN** the system MUST emit structured lifecycle evidence with cause and affected chat identifier

#### Scenario: Session reuse occurs
- **WHEN** a message arrives for an existing non-expired session
- **THEN** the system MUST emit structured evidence indicating session reuse rather than recreation
