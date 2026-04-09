---
description: Generate and store a session summary before ending
---

# Session Summary

Generate a comprehensive session summary capturing all significant work done, and store it in memory.

**Usage**: `/mvd-session-summary`

## Steps

1. Ensure the memory file exists:
// turbo
```bash
bash ./scripts/mvd-ensure.sh
```

2. Collect git changes (if in a git repo):
// turbo
```bash
git diff --name-only HEAD 2>/dev/null | head -30
```

// turbo
```bash
git diff --cached --name-only 2>/dev/null | head -30
```

3. Find recently modified files (last 30 minutes):
// turbo
```bash
find . -maxdepth 4 -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.md" -o -name "*.json" -o -name "*.py" -o -name "*.rs" -o -name "*.go" -o -name "*.toml" \) -mmin -30 ! -path "*/node_modules/*" ! -path "*/.git/*" ! -path "*/dist/*" ! -path "*/build/*" ! -path "*/target/*" 2>/dev/null | head -30
```

4. Get the git diff stat for a change summary:
// turbo
```bash
git diff HEAD --stat 2>/dev/null | head -30
```

5. Compile the session summary by reviewing:
   - All files modified (from steps 2-4)
   - Key decisions made during the session
   - Problems encountered and solutions found
   - Features added or bugs fixed
   - Any unresolved issues or next steps

6. Store the session summary:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && echo '<SESSION_SUMMARY_CONTENT>' | mvd put "$MVD_FILE" --title "Session: <BRIEF_DESCRIPTION>" --label "session" --tag "summary"
```

7. Store individual entries for important file modifications:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && echo 'Modified <filename>: <what changed>' | mvd put "$MVD_FILE" --title "Edited <filename>" --label "refactor" --tag "file-edit"
```

## Response Format
- Show a summary of what was captured
- List files modified
- List key decisions recorded
- Confirm total memory count after storage
