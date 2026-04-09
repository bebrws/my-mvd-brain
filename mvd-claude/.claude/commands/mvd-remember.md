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
- `discovery` — New information discovered
- `decision` — Important decision made
- `problem` — Problem or error encountered
- `solution` — Solution implemented
- `pattern` — Pattern recognized in code/data
- `warning` — Warning or concern noted
- `success` — Successful outcome
- `refactor` — Code refactoring done
- `bugfix` — Bug fixed
- `feature` — Feature added
- `note` — General note or observation

Store the memory:

```bash
echo '<DETAILED_CONTENT>' | mvd put ./mvd/mvd.mv2 --title "<ONE_LINE_SUMMARY>" --label "<TYPE>" --tag "manual"
```

Replace `<DETAILED_CONTENT>` with the content from `$ARGUMENTS`, `<ONE_LINE_SUMMARY>` with a concise title, and `<TYPE>` with the classified type.

Confirm storage:

```bash
mvd stats ./mvd/mvd.mv2 --json
```

## Examples
- `/mvd-remember We decided to use PostgreSQL over MongoDB for ACID compliance`
- `/mvd-remember The auth token expires after 24h and needs refresh logic`
- `/mvd-remember Bug: race condition in the queue processor when batch size > 100`

## Response Format
- Confirm what was stored with the title and type
- Show updated memory count
