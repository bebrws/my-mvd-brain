#!/usr/bin/env bash
# mvd-codex-hook-handler.sh - Codex hook handler for mvd memory capture.
#
# Reads Codex hook JSON on stdin. It fails open: all failures exit 0 so hooks
# never block the agent.

set -o pipefail

MAX_CONTENT_LENGTH=2000
MAX_TITLE_LENGTH=120
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

truncate() {
    local text="$1"
    local max_len="${2:-$MAX_CONTENT_LENGTH}"
    if [ ${#text} -gt "$max_len" ]; then
        printf '%s... [truncated]' "${text:0:$max_len}"
    else
        printf '%s' "$text"
    fi
}

json_field() {
    local payload="$1"
    local field="$2"
    local default="${3:-}"

    if command -v jq >/dev/null 2>&1; then
        printf '%s' "$payload" | jq -r ".$field // empty" 2>/dev/null
        return
    fi

    if command -v python3 >/dev/null 2>&1; then
        MVD_FIELD="$field" MVD_DEFAULT="$default" python3 -c '
import json, os, sys
try:
    data = json.load(sys.stdin)
    cur = data
    for key in os.environ["MVD_FIELD"].split("."):
        if isinstance(cur, dict):
            cur = cur.get(key)
        else:
            cur = None
            break
    if cur is None:
        print(os.environ.get("MVD_DEFAULT", ""))
    elif isinstance(cur, str):
        print(cur)
    else:
        print(json.dumps(cur, separators=(",", ":")))
except Exception:
    print(os.environ.get("MVD_DEFAULT", ""))
' 2>/dev/null <<<"$payload"
        return
    fi

    printf '%s' "$default"
}

emit_context() {
    local context="$1"
    local event_name="$2"

    if [ -z "$context" ]; then
        return
    fi

    if command -v python3 >/dev/null 2>&1; then
        MVD_CONTEXT="$context" MVD_EVENT="$event_name" python3 -c '
import json, os
event = os.environ.get("MVD_EVENT", "")
context = os.environ.get("MVD_CONTEXT", "")
print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": event,
        "additionalContext": context,
    }
}, separators=(",", ":")))
'
    fi
}

put_memory() {
    local title="$1"
    local label="$2"
    local tag="$3"
    local content="$4"

    command -v mvd >/dev/null 2>&1 || return 0

    local memory_file
    memory_file="$(bash "$SCRIPT_DIR/mvd-resolve.sh" 2>/dev/null)" || return 0

    if [ ! -f "$memory_file" ]; then
        bash "$SCRIPT_DIR/mvd-ensure.sh" >/dev/null 2>&1 || return 0
        memory_file="$(bash "$SCRIPT_DIR/mvd-resolve.sh" 2>/dev/null)" || return 0
    fi

    printf '%s' "$content" | mvd put "$memory_file" \
        --title "$(truncate "$title" "$MAX_TITLE_LENGTH")" \
        --label "$label" \
        --tag "$tag" \
        >/dev/null 2>&1 || true
}

memory_context() {
    command -v mvd >/dev/null 2>&1 || return 0

    local memory_file
    memory_file="$(bash "$SCRIPT_DIR/mvd-resolve.sh" 2>/dev/null)" || return 0

    if [ ! -f "$memory_file" ]; then
        bash "$SCRIPT_DIR/mvd-ensure.sh" >/dev/null 2>&1 || return 0
        memory_file="$(bash "$SCRIPT_DIR/mvd-resolve.sh" 2>/dev/null)" || return 0
    fi

    # Honor the workspace cwd reported by Codex so mvd's auto-scope detection
    # sees the user's actual repo + branch (not the Codex daemon cwd).
    local effective_cwd="${CWD:-$PWD}"
    [ -d "$effective_cwd" ] || effective_cwd="$PWD"

    # Strict scoping: only load context when the workspace is inside a git repo.
    # If the user opens Codex outside a repo, there is no "current repo + branch"
    # to filter on, so we skip context loading rather than falling back to all
    # repos. (The agent can still query memory explicitly during the session.)
    local repo_root branch
    repo_root="$(cd "$effective_cwd" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)"
    if [ -z "$repo_root" ]; then
        return 0
    fi
    branch="$(cd "$repo_root" 2>/dev/null && git rev-parse --abbrev-ref HEAD 2>/dev/null)"
    [ "$branch" = "HEAD" ] && branch=""  # detached HEAD → no branch filter

    # Run mvd timeline FROM the workspace so auto-detection picks up repo+branch
    # tags. With no override flags, mvd defaults to filtering to the current
    # repo + branch — exactly what we want.
    local timeline stats
    timeline="$(cd "$effective_cwd" && mvd timeline "$memory_file" --limit 12 --reverse --json 2>/dev/null | head -c 4000)"
    stats="$(mvd stats "$memory_file" --json 2>/dev/null | head -c 1000)"

    if [ -n "$timeline$stats" ]; then
        printf 'MVD persistent memory at %s (scoped to repo %s, branch %s).\nRecent timeline JSON (this repo + branch only):\n%s\nStats JSON:\n%s' \
            "$memory_file" "$repo_root" "${branch:-<detached>}" "$timeline" "$stats"
    fi
}

search_context() {
    local query="$1"
    command -v mvd >/dev/null 2>&1 || return 0
    [ -n "$query" ] || return 0

    local memory_file
    memory_file="$(bash "$SCRIPT_DIR/mvd-resolve.sh" 2>/dev/null)" || return 0
    [ -f "$memory_file" ] || return 0

    local results
    results="$(mvd find "$memory_file" --query "$query" --top-k 5 --json 2>/dev/null | head -c 3000)"
    if [ -n "$results" ]; then
        printf 'Relevant MVD memories for the current prompt:\n%s' "$results"
    fi
}

PAYLOAD="$(cat)"
[ -n "$PAYLOAD" ] || exit 0

EVENT="$(json_field "$PAYLOAD" "hook_event_name")"
[ -n "$EVENT" ] || EVENT="$(json_field "$PAYLOAD" "hookEventName")"

SESSION_ID="$(json_field "$PAYLOAD" "session_id")"
TURN_ID="$(json_field "$PAYLOAD" "turn_id")"
CWD="$(json_field "$PAYLOAD" "cwd")"
MODEL="$(json_field "$PAYLOAD" "model")"
PERMISSION_MODE="$(json_field "$PAYLOAD" "permission_mode")"

case "$EVENT" in
    SessionStart|session_start|session-start)
        SOURCE="$(json_field "$PAYLOAD" "source")"
        CONTENT="Codex session started.
Session: $SESSION_ID
Source: $SOURCE
Model: $MODEL
Directory: $CWD
Permission mode: $PERMISSION_MODE"
        put_memory "Codex session started" "session" "session-start" "$CONTENT"
        emit_context "$(memory_context)" "SessionStart"
        ;;

    UserPromptSubmit|user_prompt_submit|user-prompt-submit)
        PROMPT="$(json_field "$PAYLOAD" "prompt")"
        [ -z "$PROMPT" ] && PROMPT="$(json_field "$PAYLOAD" "user_prompt")"
        PROMPT_SHORT="$(truncate "$PROMPT" 90)"
        CONTENT="User prompt submitted.
Session: $SESSION_ID
Turn: $TURN_ID
Directory: $CWD
Prompt: $(truncate "$PROMPT" "$MAX_CONTENT_LENGTH")"
        put_memory "User prompt: $PROMPT_SHORT" "session" "user-prompt" "$CONTENT"
        emit_context "$(search_context "$PROMPT")" "UserPromptSubmit"
        ;;

    PostToolUse|post_tool_use|post-tool-use)
        TOOL_NAME="$(json_field "$PAYLOAD" "tool_name")"
        TOOL_USE_ID="$(json_field "$PAYLOAD" "tool_use_id")"
        COMMAND="$(json_field "$PAYLOAD" "tool_input.command")"
        [ -z "$COMMAND" ] && COMMAND="$(json_field "$PAYLOAD" "command")"
        RESPONSE="$(json_field "$PAYLOAD" "tool_response")"
        RESPONSE_TRUNC="$(truncate "$RESPONSE" "$MAX_CONTENT_LENGTH")"
        TITLE_TOOL="$TOOL_NAME"
        [ -z "$TITLE_TOOL" ] && TITLE_TOOL="tool"
        TITLE_DETAIL="$COMMAND"
        [ -z "$TITLE_DETAIL" ] && TITLE_DETAIL="$TOOL_USE_ID"
        CONTENT="Codex tool completed.
Session: $SESSION_ID
Turn: $TURN_ID
Tool: $TOOL_NAME
Tool use id: $TOOL_USE_ID
Directory: $CWD
Command: $COMMAND
Response:
$RESPONSE_TRUNC"
        put_memory "Codex tool: $TITLE_TOOL $(truncate "$TITLE_DETAIL" 70)" "discovery" "post-tool" "$CONTENT"
        ;;

    Stop|stop)
        REASON="$(json_field "$PAYLOAD" "reason")"
        CONTENT="Codex turn stopped.
Session: $SESSION_ID
Turn: $TURN_ID
Reason: $REASON
Model: $MODEL
Directory: $CWD"
        put_memory "Codex stop: ${REASON:-turn complete}" "session" "stop" "$CONTENT"
        ;;

    *)
        CONTENT="$(truncate "$PAYLOAD" "$MAX_CONTENT_LENGTH")"
        put_memory "Codex hook event: ${EVENT:-unknown}" "discovery" "hook" "$CONTENT"
        ;;
esac

exit 0
