---
description: Generate and store a session summary before ending
allowed-tools: Bash
---

# Session Summary

Generate a comprehensive session summary capturing all significant work done, and store it in memory.

**Usage**: `/mvd-session-summary`

Execute the following commands:

```bash
bash ./scripts/mvd-ensure.sh
```

Collect git changes (if in a git repo):

```bash
git diff --name-only HEAD 2>/dev/null | head -30
```

```bash
git diff --cached --name-only 2>/dev/null | head -30
```

Find recently modified files (last 30 minutes):

```bash
find . -maxdepth 4 -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.md" -o -name "*.json" -o -name "*.py" -o -name "*.rs" -o -name "*.go" -o -name "*.toml" \) -mmin -30 ! -path "*/node_modules/*" ! -path "*/.git/*" ! -path "*/dist/*" ! -path "*/build/*" ! -path "*/target/*" 2>/dev/null | head -30
```

Get the git diff stat:

```bash
git diff HEAD --stat 2>/dev/null | head -30
```

Compile the session summary by reviewing:
- All files modified
- Key decisions made during the session
- Problems encountered and solutions found
- Features added or bugs fixed
- Any unresolved issues or next steps

Store the session summary:

```bash
echo '<SESSION_SUMMARY_CONTENT>' | mvd put ./mvd/mvd.mv2 --title "Session: <BRIEF_DESCRIPTION>" --label "session" --tag "summary"
```

Store individual entries for important file modifications:

```bash
echo 'Modified <filename>: <what changed>' | mvd put ./mvd/mvd.mv2 --title "Edited <filename>" --label "refactor" --tag "file-edit"
```

## Response Format
- Show a summary of what was captured
- List files modified
- List key decisions recorded
- Confirm total memory count after storage
