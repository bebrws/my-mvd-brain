---
description: Ask questions about memories and get context-aware answers
---

# Memory Question

Ask questions about past work, decisions, and context using retrieval over stored memories.

**Usage**: `/mvd-ask <question>`

## Steps

1. Ensure the memory file exists:
// turbo
```bash
bash ./scripts/mvd-ensure.sh
```

2. Ask the question using retrieval (context-only mode, the agent synthesizes the answer):
// turbo
```bash
mvd ask ./mvd/mvd.mv2 --question "$ARGUMENTS" --context-only --top-k 8 --json
```

3. If the `ask` command returns no useful results, fall back to lexical search:
// turbo
```bash
mvd find ./mvd/mvd.mv2 --query "$ARGUMENTS" --top-k 8 --json
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
