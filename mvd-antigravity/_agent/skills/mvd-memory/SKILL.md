---
name: mvd-memory
description: Persistent memory system using mvd — search, query, and manage memories stored in a single portable .mv2 file. Use when the user asks about past work, wants to remember something, or needs context from previous sessions.
---

# MVD Memory System

You have access to a persistent memory system powered by `mvd`. All observations, discoveries, decisions, and learnings are stored in a single `./mvd/mvd.mv2` file.

## When to Use This Skill

Use the mvd memory system when:
- The user asks about past work, decisions, or context ("what did we do with auth?", "why did we choose X?")
- You need to recall past solutions to similar problems
- The user explicitly asks to search, store, or review memories
- You want to check if a similar problem was encountered before

## How to Execute Memory Commands

All commands use the `mvd` binary (expected in `$PATH`). The memory file is at `./mvd/mvd.mv2`.

### Ensure Memory File Exists
```bash
bash ./scripts/mvd-ensure.sh
```

### Search Memories
```bash
mvd find ./mvd/mvd.mv2 --query "<query>" --top-k 10 --json
```

Examples:
- `mvd find ./mvd/mvd.mv2 --query "authentication" --top-k 5 --json`
- `mvd find ./mvd/mvd.mv2 --query "database schema" --top-k 10 --json`

### Ask Questions (Retrieval)
```bash
mvd ask ./mvd/mvd.mv2 --question "<question>" --context-only --top-k 8 --json
```

Examples:
- `mvd ask ./mvd/mvd.mv2 --question "Why did we choose React?" --context-only --json`
- `mvd ask ./mvd/mvd.mv2 --question "What was the CORS solution?" --context-only --json`

### View Statistics
```bash
mvd stats ./mvd/mvd.mv2 --json
```

### View Recent Memories (Timeline)
```bash
mvd timeline ./mvd/mvd.mv2 --limit 20 --reverse --json
```

### Store a Memory
```bash
echo '<content>' | mvd put ./mvd/mvd.mv2 --title "<summary>" --label "<type>" --tag "<tool>"
```

### View a Specific Frame
```bash
mvd view ./mvd/mvd.mv2 <frame_id>
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

## File Location

Memory is stored at: `./mvd/mvd.mv2`

This file is:
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
