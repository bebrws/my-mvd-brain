# AGENTS.md

## MVD Memory System

You have access to a **persistent memory system** powered by `mvd`, a single-file memory engine.
All observations, decisions, discoveries, and learnings are stored in a portable `.mv2` file.

### Memory File Location

The memory file is resolved with this priority:
1. **Global**: `$HOME/mvd.mv2` — if it exists, always use this
2. **Local**: `./mvd/mvd.mv2` — per-project fallback

To resolve the path, run:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
```

Use `$MVD_FILE` in all subsequent `mvd` commands within the conversation.

### Memory File Setup

At the **beginning of every conversation**, do the following SILENTLY (no need to tell the user):

1. Run the ensure script to create the memory file if it doesn't exist:
   ```bash
   bash ./scripts/mvd-ensure.sh
   ```
2. Resolve the memory file path:
   ```bash
   MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
   ```
3. Load recent context by running:
   ```bash
   mvd timeline "$MVD_FILE" --limit 20 --reverse --json 2>/dev/null
   ```
4. Check memory stats:
   ```bash
   mvd stats "$MVD_FILE" --json 2>/dev/null
   ```

Use the returned context to inform your work. Reference relevant past memories when they apply to the current task.

### Automatic Memory Capture

After performing **significant work**, proactively store an observation. Significant work includes:

- Creating or editing files
- Running commands that produce meaningful output
- Discovering bugs or issues
- Making architectural decisions
- Finding solutions to problems
- Completing features or fixes

To store an observation, run:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
echo '<CONTENT>' | mvd put "$MVD_FILE" --title "<SUMMARY>" --label "<TYPE>" --tag "<TOOL>"
```

Where:
- `<CONTENT>` is a compressed description of what happened (key facts only, 2-5 sentences)
- `<SUMMARY>` is a one-line summary (e.g., "Fixed CORS bug in auth middleware")
- `<TYPE>` is one of: `discovery`, `decision`, `problem`, `solution`, `pattern`, `warning`, `success`, `refactor`, `bugfix`, `feature`
- `<TOOL>` is what triggered it (e.g., `file-edit`, `command`, `search`, `web-fetch`)

#### Capture Guidelines

- **DO capture**: Key decisions, bugs found/fixed, important file changes, architectural choices, error resolutions
- **DO NOT capture**: Trivial reads, listing directories, routine formatting, or anything the user explicitly asks you not to store
- **Compress aggressively**: Store key facts, not raw output. A 500-line file read → "Read auth.rs: implements JWT validation with RS256, expiry check, role-based claims"
- **Avoid duplicates**: Don't re-store the same observation within a single conversation

### Session Summary

Before the conversation ends (when wrapping up or the user says goodbye), generate a session summary:

1. Collect git changes if in a git repo:
   ```bash
   git diff --name-only HEAD 2>/dev/null | head -20
   ```
2. Store a summary:
   ```bash
   MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
   echo '<SESSION_SUMMARY>' | mvd put "$MVD_FILE" --title "Session summary: <BRIEF_DESCRIPTION>" --label "session" --tag "summary"
   ```

The session summary should include what was accomplished, key decisions, files modified, and any unresolved issues.

### Memory Commands

When the user asks to query their memory, use these commands:

**Search memories:**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "<query>" --top-k 10 --json
```

**Ask questions (retrieval):**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd ask "$MVD_FILE" --question "<question>" --context-only --top-k 8 --json
```

**View statistics:**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --json
```

**View recent memories (timeline):**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd timeline "$MVD_FILE" --limit 20 --reverse --json
```

**Store a memory:**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && echo '<content>' | mvd put "$MVD_FILE" --title "<summary>" --label "<type>" --tag "<tool>"
```

**View a specific frame:**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd view "$MVD_FILE" <frame_id>
```

### Memory Skill — When to Use

Use the mvd memory system proactively when:
- The user asks about past work, decisions, or context
- You need to recall past solutions to similar problems
- The user explicitly asks to search, store, or review memories
- You want to check if a similar problem was encountered before

### Memory Types

When storing memories, classify them with the appropriate label:
- **discovery** — New information discovered
- **decision** — Important decision made
- **problem** — Problem or error encountered
- **solution** — Solution implemented
- **pattern** — Pattern recognized in code/data
- **warning** — Warning or concern noted
- **success** — Successful outcome
- **refactor** — Code refactoring done
- **bugfix** — Bug fixed
- **feature** — Feature added
- **session** — Session summary
