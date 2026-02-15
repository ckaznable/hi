## MODIFIED Requirements

### Requirement: Skill file format
Each skill SHALL be a Markdown file (`.md`) in the skills directory. The file name (without extension) SHALL be the skill name. The file SHALL support optional YAML frontmatter with a `description` field. The file content (after frontmatter) SHALL be the skill's system prompt / guidance text. If no frontmatter is provided, the file name SHALL be used as the description.

#### Scenario: Load skill with frontmatter
- **WHEN** the file `skills/coding-assistant.md` exists with frontmatter `---\ndescription: Expert coding guidance\n---\nYou are a coding expert...`
- **THEN** the system SHALL load it as a skill named `coding-assistant` with description `Expert coding guidance` and the body as skill text

#### Scenario: Load skill without frontmatter
- **WHEN** the file `skills/translator.md` exists with content `You are a translator...` and no frontmatter
- **THEN** the system SHALL load it as a skill named `translator` with the file name `translator` as the description
