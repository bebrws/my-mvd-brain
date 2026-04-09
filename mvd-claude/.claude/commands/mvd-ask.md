---
description: Ask questions about memories and get context-aware answers
argument-hint: <question>
allowed-tools: Bash
---

# Memory Question

Ask questions about past work, decisions, and context using retrieval over stored memories.

**Usage**: `/mvd-ask <question>`

Execute the following commands:

```bash
bash ./scripts/mvd-ensure.sh
```

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd ask "$MVD_FILE" --question "$ARGUMENTS" --context-only --top-k 8 --json
```

If the ask command returns no useful results, fall back to lexical search:

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "$ARGUMENTS" --top-k 8 --json
```

## Examples
- `/mvd-ask Why did we choose React?` — Get context about technology decisions
- `/mvd-ask What was the CORS solution?` — Recall specific solutions
- `/mvd-ask How did we fix the authentication bug?` — Get details about past fixes

## Response Format
- Synthesize a context-aware answer based on the retrieved memories
- Reference specific memories when applicable
- Include timestamps for referenced information
- If no relevant memories found, say so clearly
