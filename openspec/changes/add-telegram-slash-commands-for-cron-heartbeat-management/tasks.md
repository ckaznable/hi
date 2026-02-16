# Tasks — add-telegram-slash-commands-for-cron-heartbeat-management

## 1. Add config accessor to SessionManager — DONE
- Added `pub fn config(&self) -> &ModelConfig` to `SessionManager`

## 2. Add Telegram slash commands — DONE
- `/cron` — lists all configured schedules (name, cron, model, prompt preview)
- `/cron add <name> <min> <hour> <dom> <mon> <dow> <prompt>` — adds a schedule, persists to schedules.json
- `/cron remove <name>` — removes a schedule by name, persists
- `/heartbeat` — shows heartbeat status (enabled, interval, model, prompt)
- `/mcp` — lists configured MCP servers (name, transport type: stdio/http)
- `/help` updated to include all new commands
- All commands bypass LLM — pure deterministic handlers

## 3. Add formatting helpers — DONE
- `format_schedules()` — formats schedule list
- `format_heartbeat()` — formats heartbeat status
- `format_mcp_servers()` — formats MCP server list
- `handle_cron_command()` — routes cron subcommands
- `handle_cron_add()` / `handle_cron_remove()` — cron CRUD with validation

## 4. Tests — DONE
- 12 new tests for formatting and command parsing
- All existing tests preserved

## 5. Verification — DONE
- `cargo check --workspace` — clean, 0 warnings
- `cargo test --workspace` — 200 tests, 0 failures
