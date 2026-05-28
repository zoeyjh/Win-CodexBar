# R-2: Session Log Sources

## Claude CLI
- **Found:** `~/.claude/history.jsonl` + per-project JSONL files
- **Location:** `C:\Users\zoey.j.han\.claude\history.jsonl`
- **Per-project:** `~/.claude/projects/<project-id>/<session-id>.jsonl`
- **Decision:** ✅ Available. Can parse for Card 2.

## Codex CLI
- **Found:** Needs investigation. No `~/.codex` directory found on this machine.
- **Decision:** TBD — may not be available if Codex CLI doesn't log locally.

## Copilot CLI
- **Found:** No `~/.copilot-cli` or `%LOCALAPPDATA%\github-copilot` found.
- **Decision:** ❌ Not available. Card 2 will show "Session log not available" for Copilot.

## Impact on Card 2
- Claude: Show session data from local JSONL files
- Codex: TBD (show placeholder if no local logs)
- Copilot: "Session log not available for this provider"
