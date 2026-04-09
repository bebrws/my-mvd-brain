---
description: Show memory statistics and storage information
allowed-tools: Bash
---

# Memory Statistics

Show statistics about the persistent memory file.

Execute the following commands:

```bash
bash ./scripts/mvd-ensure.sh
```

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --json
```

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && ls -lh "$MVD_FILE" 2>/dev/null | awk '{print $5}'
```

## Response Format
- Convert any Unix timestamps to human-readable format (e.g., "2h ago", "3d ago")
- Present key stats in a clean table: total frames, file size, latest memory date
- If the file was just created, tell the user memories will appear as they work
