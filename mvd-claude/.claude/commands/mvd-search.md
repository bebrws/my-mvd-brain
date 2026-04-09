---
description: Search memories for specific content or patterns
argument-hint: <query>
allowed-tools: Bash
---

# Memory Search

Search through persistent memories for specific content, patterns, or keywords.

**Usage**: `/mvd-search <query>`

Execute the following commands:

```bash
bash ./scripts/mvd-ensure.sh
```

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd find "$MVD_FILE" --query "$ARGUMENTS" --top-k 10 --json
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
