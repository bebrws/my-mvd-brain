---
name: mvd-memory
description: Persistent memory system using mvd — search, query, and manage memories stored in a single portable .mv2 file. Use when the user asks about past work, wants to remember something, or needs context from previous sessions.
---

# MVD Memory System

You have access to a persistent memory system powered by `mvd`. All observations, discoveries, decisions, and learnings are stored in a single `.mv2` file.

## Memory File Location

The memory file is resolved with this priority:
1. **Global**: `$HOME/mvd.mv2` — if it exists, always use this
2. **Local**: `./mvd/mvd.mv2` — per-project fallback

Resolve the path before any operation:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
```

## When to Use This Skill

Use the mvd memory system when:
- The user asks about past work, decisions, or context ("what did we do with auth?", "why did we choose X?")
- You need to recall past solutions to similar problems
- The user explicitly asks to search, store, or review memories
- You want to check if a similar problem was encountered before

## How to Execute Memory Commands

All commands use the `mvd` binary (expected in `$PATH`). Always resolve the file path first.

### Ensure Memory File Exists
```bash
bash ./scripts/mvd-ensure.sh
```

### Search Memories
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "<query>" --top-k 10 --json
```

### Ask Questions (Retrieval)
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd ask "$MVD_FILE" --question "<question>" --context-only --top-k 8 --json
```

### View Statistics
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --json
```

### View Recent Memories (Timeline)
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd timeline "$MVD_FILE" --limit 20 --reverse --json
```

### Store a Memory
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && echo '<content>' | mvd put "$MVD_FILE" --title "<summary>" --label "<type>" --tag "<tool>"
```

### View a Specific Frame
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd view "$MVD_FILE" <frame_id>
```

## Memory Types

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

## File Properties

The `.mv2` file is:
- **Portable** — Copy it anywhere, share with teammates
- **Git-friendly** — Commit to version control
- **Self-contained** — Everything in ONE file
- **Searchable** — Instant lexical and vector search

## Tips

1. **Start of session**: Recent memories are loaded via rules and used as context
2. **During work**: Observations are captured proactively after significant tool use
3. **Searching**: Use natural language queries to find relevant past context
4. **Session end**: A summary is generated before the conversation ends
5. **Manual storage**: Use the `/mvd-remember` workflow to explicitly store something
