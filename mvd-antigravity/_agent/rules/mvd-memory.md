# MVD Memory System — Agent Rules

You have access to a **persistent memory system** powered by `mvd`, a single-file memory engine.
All observations, decisions, discoveries, and learnings are stored in a portable `.mv2` file at `./mvd/mvd.mv2`.

## Memory File Setup

At the **beginning of every conversation**, do the following SILENTLY (no need to tell the user):

1. Run the ensure script to create the memory file if it doesn't exist:
   ```bash
   bash ./scripts/mvd-ensure.sh
   ```
2. Load recent context by running:
   ```bash
   mvd timeline ./mvd/mvd.mv2 --limit 20 --reverse --json 2>/dev/null
   ```
3. Check memory stats:
   ```bash
   mvd stats ./mvd/mvd.mv2 --json 2>/dev/null
   ```

Use the returned context to inform your work. Reference relevant past memories when they apply to the current task.

## Automatic Memory Capture

After performing **significant work**, proactively store an observation. Significant work includes:

- Creating or editing files
- Running commands that produce meaningful output
- Discovering bugs or issues
- Making architectural decisions
- Finding solutions to problems
- Completing features or fixes

To store an observation, run:
```bash
echo '<CONTENT>' | mvd put ./mvd/mvd.mv2 --title "<SUMMARY>" --label "<TYPE>" --tag "<TOOL>"
```

Where:
- `<CONTENT>` is a compressed description of what happened (key facts only, 2-5 sentences)
- `<SUMMARY>` is a one-line summary (e.g., "Fixed CORS bug in auth middleware")
- `<TYPE>` is one of: `discovery`, `decision`, `problem`, `solution`, `pattern`, `warning`, `success`, `refactor`, `bugfix`, `feature`
- `<TOOL>` is what triggered it (e.g., `file-edit`, `command`, `search`, `web-fetch`)

### Capture Guidelines

- **DO capture**: Key decisions, bugs found/fixed, important file changes, architectural choices, error resolutions
- **DO NOT capture**: Trivial reads, listing directories, routine formatting, or anything the user explicitly asks you not to store
- **Compress aggressively**: Store key facts, not raw output. A 500-line file read → "Read auth.rs: implements JWT validation with RS256, expiry check, role-based claims"
- **Avoid duplicates**: Don't re-store the same observation within a single conversation

## Session Summary

Before the conversation ends (when wrapping up or the user says goodbye), generate a session summary:

1. Collect git changes if in a git repo:
   ```bash
   git diff --name-only HEAD 2>/dev/null | head -20
   ```
2. Store a summary:
   ```bash
   echo '<SESSION_SUMMARY>' | mvd put ./mvd/mvd.mv2 --title "Session summary: <BRIEF_DESCRIPTION>" --label "session" --tag "summary"
   ```

The session summary should include:
- What was accomplished
- Key decisions made
- Files modified
- Any unresolved issues or next steps

## Available Workflows

You can tell the user about these available commands:
- `/mvd-stats` — View memory statistics
- `/mvd-search <query>` — Search memories
- `/mvd-ask <question>` — Ask questions about past work
- `/mvd-recent [count]` — View recent memories
- `/mvd-remember` — Manually store a memory
- `/mvd-session-summary` — Generate session summary
