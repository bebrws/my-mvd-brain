---
description: Show recent memories and activity timeline
---

# Recent Memories

Display the most recent memories and activity from the persistent memory store.

**Usage**: `/mvd-recent [count]`

## Steps

1. Ensure the memory file exists:
// turbo
```bash
bash ./scripts/mvd-ensure.sh
```

2. Show recent memories (default 20, or user-specified count):
// turbo
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd timeline "$MVD_FILE" --limit ${1:-20} --reverse --json
```

## Examples
- `/mvd-recent` — Show 20 most recent memories (default)
- `/mvd-recent 10` — Show 10 most recent memories
- `/mvd-recent 50` — Show 50 most recent memories

## Response Format
- Display memories in reverse chronological order (newest first)
- Convert timestamps to human-readable format (e.g., "2 hours ago", "yesterday")
- Group by session or time period when helpful
- Show title, type/label, and a snippet of each memory
