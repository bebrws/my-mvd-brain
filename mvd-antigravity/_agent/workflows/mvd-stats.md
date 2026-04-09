---
description: Show memory statistics and storage information
---

# Memory Statistics

Show statistics about the persistent memory file.

## Steps

1. Ensure the memory file exists:
// turbo
```bash
bash ./scripts/mvd-ensure.sh
```

2. Resolve memory file path and get statistics:
// turbo
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --json
```

3. Get the file size:
// turbo
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && ls -lh "$MVD_FILE" 2>/dev/null | awk '{print $5}'
```

## Response Format
- Convert any Unix timestamps to human-readable format (e.g., "2h ago", "3d ago")
- Present key stats in a clean table: total frames, file size, latest memory date
- If the file was just created, tell the user memories will appear as they work
