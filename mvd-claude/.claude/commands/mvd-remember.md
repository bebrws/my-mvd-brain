---
description: Manually store a memory or observation
argument-hint: <what to remember>
allowed-tools: Bash
---

# Remember

Manually store a specific memory, observation, or note into the persistent memory system.

**Usage**: `/mvd-remember <what to remember>`

Execute the ensure script first:

```bash
bash ./scripts/mvd-ensure.sh
```

Then classify the memory type. Choose the most appropriate from:
- `discovery`, `decision`, `problem`, `solution`, `pattern`, `warning`, `success`, `refactor`, `bugfix`, `feature`, `note`

Store the memory:

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && echo '<DETAILED_CONTENT>' | mvd put "$MVD_FILE" --title "<ONE_LINE_SUMMARY>" --label "<TYPE>" --tag "manual"
```

Replace `<DETAILED_CONTENT>` with the content from `$ARGUMENTS`, `<ONE_LINE_SUMMARY>` with a concise title, and `<TYPE>` with the classified type.

Confirm storage:

```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --json
```

## Examples
- `/mvd-remember We decided to use PostgreSQL over MongoDB for ACID compliance`
- `/mvd-remember The auth token expires after 24h and needs refresh logic`
- `/mvd-remember Bug: race condition in the queue processor when batch size > 100`

## Response Format
- Confirm what was stored with the title and type
- Show updated memory count
