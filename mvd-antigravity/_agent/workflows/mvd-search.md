---
description: Search memories for specific content or patterns
---

# Memory Search

Search through persistent memories for specific content, patterns, or keywords.

**Usage**: `/mvd-search <query>`

## Steps

1. Ensure the memory file exists:
// turbo
```bash
bash ./scripts/mvd-ensure.sh
```

2. Search memories with the user's query:
// turbo
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "$1" --top-k 10 --json
```

If the user provides a custom limit, use it instead of 10:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "$1" --top-k $2 --json
```

## Examples
- `/mvd-search authentication` — Find memories related to authentication
- `/mvd-search "database schema"` — Search for exact phrase
- `/mvd-search API errors` — Find memories about API errors

## Response Format
- Show matching memories with relevance scores
- Include any timestamps (convert to human-readable format)
- Highlight matched keywords in context
- If no results found, suggest alternative search terms
