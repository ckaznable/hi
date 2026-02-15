## ADDED Requirements

### Requirement: ReadSkills tool
The system SHALL provide a `ReadSkillsTool` implementing rig's `Tool` trait that returns all available skills with their names and descriptions.

#### Scenario: List all skills
- **WHEN** the LLM calls `ReadSkillsTool` with no arguments
- **THEN** the tool SHALL return a list of all loaded skills with their names and descriptions

#### Scenario: No skills available
- **WHEN** the LLM calls `ReadSkillsTool` and no skills are loaded
- **THEN** the tool SHALL return an empty list
