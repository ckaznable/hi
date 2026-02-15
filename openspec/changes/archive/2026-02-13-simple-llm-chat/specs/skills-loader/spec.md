## ADDED Requirements

### Requirement: Skills directory location
The system SHALL read skill files from `ProjectDirs::config_dir("hi")/skills/` directory.

#### Scenario: Skills directory exists
- **WHEN** the application starts and the `skills/` directory exists in config_dir
- **THEN** the system SHALL scan the directory for `.md` files

#### Scenario: Skills directory does not exist
- **WHEN** the application starts and the `skills/` directory does not exist
- **THEN** the system SHALL proceed with no skills loaded (empty skills list)

### Requirement: Skill file format
Each skill SHALL be a Markdown file (`.md`) in the skills directory. The file name (without extension) SHALL be the skill name. The file content SHALL be the skill's system prompt / guidance text.

#### Scenario: Load skill from file
- **WHEN** the file `skills/coding-assistant.md` exists with content "You are a coding expert..."
- **THEN** the system SHALL load it as a skill named `coding-assistant` with the file content as the skill text

### Requirement: Skills injection into agent context
All loaded skills SHALL be appended to the agent's preamble as additional context, after the user-configured preamble.

#### Scenario: Agent with skills
- **WHEN** the config has a preamble and 2 skills are loaded
- **THEN** the agent SHALL be configured with the config preamble followed by all skill contents concatenated

#### Scenario: No skills loaded
- **WHEN** no skill files exist in the skills directory
- **THEN** the agent SHALL use only the config preamble without any additional context

### Requirement: Skills loaded at startup
Skills SHALL be loaded once at application startup. Changes to skill files SHALL require restarting the application to take effect.

#### Scenario: Startup loading
- **WHEN** the application starts
- **THEN** the system SHALL read all `.md` files from the skills directory and make them available to the agent
