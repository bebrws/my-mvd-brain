#!/usr/bin/env bash
# mvd-hook-handler.sh — Unified Cursor hook handler for mvd memory capture
#
# Called by Cursor hooks for every lifecycle event. Reads JSON payload from
# stdin, extracts relevant data, and stores a memory frame via `mvd put`.
#
# This script is designed to:
#   - Never block the agent (fire-and-forget, always exits 0)
#   - Compress aggressively (truncate long outputs)
#   - Tag frames with conversation_id for session filtering
#   - Handle all hook events via a single entry point

set -o pipefail

# ── Configuration ──────────────────────────────────────────────────────────
MAX_CONTENT_LENGTH=2000   # Truncate content beyond this many chars
MAX_TITLE_LENGTH=120      # Truncate titles beyond this
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ── Resolve memory file ───────────────────────────────────────────────────
MVD_FILE="$(bash "$SCRIPT_DIR/mvd-resolve-global.sh" 2>/dev/null)"

# Ensure mvd is available
if ! command -v mvd &>/dev/null; then
    exit 0
fi

# Ensure memory file exists; create if needed
if [ ! -f "$MVD_FILE" ]; then
    mvd create "$MVD_FILE" 2>/dev/null || exit 0
fi

# ── Read JSON payload from stdin ──────────────────────────────────────────
PAYLOAD=$(cat)

if [ -z "$PAYLOAD" ]; then
    exit 0
fi

# ── JSON field extraction ─────────────────────────────────────────────────
# Try jq first, fall back to python3
extract_field() {
    local field="$1"
    local default="${2:-}"
    local value

    if command -v jq &>/dev/null; then
        value=$(echo "$PAYLOAD" | jq -r ".$field // empty" 2>/dev/null)
    elif command -v python3 &>/dev/null; then
        value=$(echo "$PAYLOAD" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    keys = '$field'.split('.')
    v = d
    for k in keys:
        if isinstance(v, dict):
            v = v.get(k)
        else:
            v = None
            break
    if v is not None:
        print(v if isinstance(v, str) else json.dumps(v))
except: pass
" 2>/dev/null)
    fi

    echo "${value:-$default}"
}

# ── Truncation helper ─────────────────────────────────────────────────────
truncate() {
    local text="$1"
    local max_len="${2:-$MAX_CONTENT_LENGTH}"
    if [ ${#text} -gt "$max_len" ]; then
        echo "${text:0:$max_len}... [truncated]"
    else
        echo "$text"
    fi
}

# ── Extract common fields ─────────────────────────────────────────────────
EVENT=$(extract_field "hook_event_name")
CONVERSATION_ID=$(extract_field "conversation_id")
MODEL=$(extract_field "model")
WORKSPACE=$(extract_field "workspace_roots[0]" "unknown")

if [ -z "$EVENT" ]; then
    exit 0
fi

# ── Route by event type ───────────────────────────────────────────────────
case "$EVENT" in

    sessionStart)
        SESSION_ID=$(extract_field "session_id")
        COMPOSER_MODE=$(extract_field "composer_mode" "agent")
        IS_BG=$(extract_field "is_background_agent" "false")

        TITLE="Session started: ${COMPOSER_MODE} mode"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")
        CONTENT="Session ${SESSION_ID} started.
Model: ${MODEL}
Workspace: ${WORKSPACE}
Mode: ${COMPOSER_MODE}
Background agent: ${IS_BG}"
        LABEL="session"
        TAG="session-start"
        ;;

    sessionEnd)
        SESSION_ID=$(extract_field "session_id")
        REASON=$(extract_field "reason" "unknown")
        DURATION_MS=$(extract_field "duration_ms" "0")
        ERROR_MSG=$(extract_field "error_message")

        # Convert ms to human-readable
        if [ "$DURATION_MS" -gt 0 ] 2>/dev/null; then
            DURATION_SEC=$((DURATION_MS / 1000))
            DURATION_MIN=$((DURATION_SEC / 60))
            DURATION_REMAINDER=$((DURATION_SEC % 60))
            if [ "$DURATION_MIN" -gt 0 ]; then
                DURATION_STR="${DURATION_MIN}m ${DURATION_REMAINDER}s"
            else
                DURATION_STR="${DURATION_SEC}s"
            fi
        else
            DURATION_STR="unknown"
        fi

        TITLE="Session ended: ${REASON} (${DURATION_STR})"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")
        CONTENT="Session ${SESSION_ID} ended.
Reason: ${REASON}
Duration: ${DURATION_STR}
Model: ${MODEL}
Workspace: ${WORKSPACE}"
        if [ -n "$ERROR_MSG" ]; then
            CONTENT="${CONTENT}
Error: ${ERROR_MSG}"
        fi
        LABEL="session"
        TAG="session-end"
        ;;

    afterShellExecution)
        CMD=$(extract_field "command" "unknown command")
        CMD_OUTPUT=$(extract_field "output")
        CWD=$(extract_field "cwd")
        EXIT_CODE=$(extract_field "exit_code")
        DURATION=$(extract_field "duration")

        # Build a short summary of the command
        CMD_SHORT=$(truncate "$CMD" 80)
        TITLE="Shell: ${CMD_SHORT}"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")

        # Truncate output for storage
        CMD_OUTPUT_TRUNC=$(truncate "$CMD_OUTPUT" "$MAX_CONTENT_LENGTH")

        CONTENT="Command: ${CMD}
Directory: ${CWD}
Exit code: ${EXIT_CODE}
Duration: ${DURATION}ms
Output:
${CMD_OUTPUT_TRUNC}"
        LABEL="command"
        TAG="shell"
        ;;

    afterMCPExecution)
        TOOL_NAME=$(extract_field "tool_name" "unknown-tool")
        TOOL_INPUT=$(extract_field "tool_input")
        # Cursor sends result_json; older payloads may use tool_output
        TOOL_OUTPUT=$(extract_field "result_json")
        if [ -z "$TOOL_OUTPUT" ]; then
            TOOL_OUTPUT=$(extract_field "tool_output")
        fi
        SERVER_NAME=$(extract_field "server_name")
        DURATION=$(extract_field "duration")

        TOOL_INPUT_TRUNC=$(truncate "$TOOL_INPUT" 500)
        TOOL_OUTPUT_TRUNC=$(truncate "$TOOL_OUTPUT" "$MAX_CONTENT_LENGTH")

        TITLE="MCP tool: ${TOOL_NAME}"
        if [ -n "$SERVER_NAME" ]; then
            TITLE="MCP tool: ${SERVER_NAME}/${TOOL_NAME}"
        fi
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")

        CONTENT="Tool: ${TOOL_NAME}
Server: ${SERVER_NAME}
Duration: ${DURATION}ms
Input: ${TOOL_INPUT_TRUNC}
Output:
${TOOL_OUTPUT_TRUNC}"
        LABEL="mcp-tool"
        TAG="mcp"
        ;;

    afterFileEdit)
        FILE_PATH=$(extract_field "file_path" "unknown file")
        # Extract just the filename for the title
        FILE_NAME=$(basename "$FILE_PATH" 2>/dev/null || echo "$FILE_PATH")

        # Get edit details — edits is an array
        if command -v jq &>/dev/null; then
            EDIT_COUNT=$(echo "$PAYLOAD" | jq '.edits | length' 2>/dev/null || echo "0")
            EDIT_SUMMARY=$(echo "$PAYLOAD" | jq -r '
                .edits[:3][] |
                "- Replaced: \(.old_string // "?" | .[0:80]) → \(.new_string // "?" | .[0:80])"
            ' 2>/dev/null || echo "")
        elif command -v python3 &>/dev/null; then
            EDIT_SUMMARY=$(echo "$PAYLOAD" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    edits = d.get('edits', [])
    print(f'{len(edits)} edit(s)')
    for e in edits[:3]:
        old = (e.get('old_string','') or '')[:80]
        new = (e.get('new_string','') or '')[:80]
        print(f'- Replaced: {old} → {new}')
except: pass
" 2>/dev/null || echo "")
            EDIT_COUNT="?"
        else
            EDIT_SUMMARY=""
            EDIT_COUNT="?"
        fi

        TITLE="Edited: ${FILE_NAME}"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")

        CONTENT="File: ${FILE_PATH}
Edits: ${EDIT_COUNT}
${EDIT_SUMMARY}"
        CONTENT=$(truncate "$CONTENT" "$MAX_CONTENT_LENGTH")
        LABEL="file-edit"
        TAG="edit"
        ;;

    afterAgentResponse)
        AGENT_TEXT=$(extract_field "text")

        # Skip very short responses (acknowledgements, etc.)
        if [ ${#AGENT_TEXT} -lt 50 ]; then
            exit 0
        fi

        # Use first line as title, truncated
        FIRST_LINE=$(echo "$AGENT_TEXT" | head -1)
        TITLE="Agent response: $(truncate "$FIRST_LINE" 80)"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")

        CONTENT=$(truncate "$AGENT_TEXT" "$MAX_CONTENT_LENGTH")
        LABEL="agent-response"
        TAG="response"
        ;;

    stop)
        STATUS=$(extract_field "status" "unknown")
        LOOP_COUNT=$(extract_field "loop_count" "0")

        TITLE="Agent stopped: ${STATUS}"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")
        CONTENT="Agent loop ended.
Status: ${STATUS}
Loop count: ${LOOP_COUNT}
Model: ${MODEL}
Workspace: ${WORKSPACE}"
        LABEL="session"
        TAG="stop"
        ;;

    preCompact)
        TRIGGER=$(extract_field "trigger" "auto")
        USAGE=$(extract_field "context_usage_percent" "?")
        TOKENS=$(extract_field "context_tokens" "?")
        WINDOW=$(extract_field "context_window_size" "?")
        MSG_COUNT=$(extract_field "message_count" "?")

        TITLE="Context compaction: ${USAGE}% used (${TRIGGER})"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")
        CONTENT="Context window compaction triggered.
Trigger: ${TRIGGER}
Usage: ${USAGE}% (${TOKENS}/${WINDOW} tokens)
Messages: ${MSG_COUNT}"
        LABEL="discovery"
        TAG="compaction"
        ;;

    *)
        # Unknown event — capture it anyway for debugging
        TITLE="Hook event: ${EVENT}"
        TITLE=$(truncate "$TITLE" "$MAX_TITLE_LENGTH")
        CONTENT=$(truncate "$PAYLOAD" "$MAX_CONTENT_LENGTH")
        LABEL="discovery"
        TAG="hook"
        ;;
esac

# ── Store the memory frame ────────────────────────────────────────────────
# Build tags: always include the event tag, and add conversation_id if present
TAGS="$TAG"

# Silence mvd stdout — command hooks must print valid JSON on stdout for Cursor
echo "$CONTENT" | mvd put "$MVD_FILE" \
    --title "$TITLE" \
    --label "$LABEL" \
    --tag "$TAGS" \
    >/dev/null 2>&1 || true

# Command hooks expect JSON on stdout (see Cursor docs)
echo '{}'

# Always exit 0 — never block the agent
exit 0
