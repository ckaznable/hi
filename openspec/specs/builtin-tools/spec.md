## ADDED Requirements

### Requirement: Bash execution tool
The system SHALL provide a `BashTool` implementing rig's `Tool` trait that executes a bash command and returns stdout, stderr, and exit code.

#### Scenario: Successful command execution
- **WHEN** the LLM calls `BashTool` with `{ "command": "echo hello" }`
- **THEN** the tool SHALL execute the command via `tokio::process::Command` and return `{ "stdout": "hello\n", "stderr": "", "exit_code": 0 }`

#### Scenario: Failed command
- **WHEN** the LLM calls `BashTool` with `{ "command": "nonexistent_cmd" }`
- **THEN** the tool SHALL return the stderr output and a non-zero exit code

### Requirement: List files tool
The system SHALL provide a `ListFilesTool` implementing rig's `Tool` trait that lists entries in a directory.

#### Scenario: List directory contents
- **WHEN** the LLM calls `ListFilesTool` with `{ "path": "/tmp" }`
- **THEN** the tool SHALL return a list of file and directory names in the specified path

#### Scenario: Non-existent directory
- **WHEN** the LLM calls `ListFilesTool` with a path that does not exist
- **THEN** the tool SHALL return an error indicating the directory was not found

### Requirement: Read file tool
The system SHALL provide a `ReadFileTool` implementing rig's `Tool` trait that reads the content of a file.

#### Scenario: Read existing file
- **WHEN** the LLM calls `ReadFileTool` with `{ "path": "/tmp/test.txt" }`
- **THEN** the tool SHALL return the file content as a string

#### Scenario: Non-existent file
- **WHEN** the LLM calls `ReadFileTool` with a path to a file that does not exist
- **THEN** the tool SHALL return an error indicating the file was not found

### Requirement: Write file tool
The system SHALL provide a `WriteFileTool` implementing rig's `Tool` trait that writes content to a file.

#### Scenario: Write to file
- **WHEN** the LLM calls `WriteFileTool` with `{ "path": "/tmp/output.txt", "content": "hello world" }`
- **THEN** the tool SHALL write the content to the file and return `{ "success": true }`

#### Scenario: Write to read-only path
- **WHEN** the LLM calls `WriteFileTool` with a path that is not writable
- **THEN** the tool SHALL return an error indicating permission denied

### Requirement: ReadSkills tool
The system SHALL provide a `ReadSkillsTool` implementing rig's `Tool` trait that returns all available skills with their names and descriptions.

#### Scenario: List all skills
- **WHEN** the LLM calls `ReadSkillsTool` with no arguments
- **THEN** the tool SHALL return a list of all loaded skills with their names and descriptions

#### Scenario: No skills available
- **WHEN** the LLM calls `ReadSkillsTool` and no skills are loaded
- **THEN** the tool SHALL return an empty list

### Requirement: Tool registration
All built-in tools SHALL be registered with the rig agent via the `.tool()` builder method, making them available for LLM function calling.

#### Scenario: Agent uses tools
- **WHEN** the agent is built with all built-in tools registered
- **THEN** the LLM SHALL be able to invoke any of the registered tools during a conversation
