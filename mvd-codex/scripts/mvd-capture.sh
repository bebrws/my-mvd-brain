#!/usr/bin/env bash
# mvd-capture.sh — Capture a tool observation as a memory frame
#
# Usage: mvd-capture.sh <tool_name> <summary> [content_file]
#
# Arguments:
#   $1 — Tool name (e.g., "file-edit", "command", "search", "web-fetch")
#   $2 — Summary of what happened (one line)
#   $3 — Optional: path to a file containing the detailed content
#         If not provided, reads from stdin

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"

if [ $# -lt 2 ]; then
    echo "Usage: mvd-capture.sh <tool_name> <summary> [content_file]" >&2
    echo "" >&2
    echo "  tool_name: file-edit, command, search, web-fetch, grep, etc." >&2
    echo "  summary:   One-line description of the observation" >&2
    echo "  content:   Detailed content (stdin or file path)" >&2
    exit 1
fi

TOOL_NAME="$1"
SUMMARY="$2"
CONTENT_FILE="${3:-}"

# Ensure memory file exists
if [ ! -f "${MEMORY_FILE}" ]; then
    bash "${SCRIPT_DIR}/mvd-ensure.sh"
    MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
fi

# Classify the observation type based on tool and content
classify_type() {
    local tool="$1"
    local summary_lower
    summary_lower=$(echo "$2" | tr '[:upper:]' '[:lower:]')

    if echo "${summary_lower}" | grep -qE '(error|failed|crash|panic|bug|broken|issue)'; then
        echo "problem"
        return
    fi

    if echo "${summary_lower}" | grep -qE '(fix|resolve|solution|solved|repair)'; then
        echo "bugfix"
        return
    fi

    case "${tool}" in
        file-edit|edit|write)
            if echo "${summary_lower}" | grep -qE '(refactor|rename|move|reorganize)'; then
                echo "refactor"
            elif echo "${summary_lower}" | grep -qE '(add|create|implement|feature)'; then
                echo "feature"
            else
                echo "refactor"
            fi
            ;;
        command|bash)
            echo "discovery"
            ;;
        search|grep|find)
            echo "discovery"
            ;;
        web-fetch|web-search)
            echo "discovery"
            ;;
        *)
            echo "discovery"
            ;;
    esac
}

TYPE=$(classify_type "${TOOL_NAME}" "${SUMMARY}")

# Store the observation
if [ -n "${CONTENT_FILE}" ] && [ -f "${CONTENT_FILE}" ]; then
    mvd put "${MEMORY_FILE}" --title "${SUMMARY}" --label "${TYPE}" --tag "${TOOL_NAME}" --input "${CONTENT_FILE}"
else
    mvd put "${MEMORY_FILE}" --title "${SUMMARY}" --label "${TYPE}" --tag "${TOOL_NAME}" < /dev/null
fi

echo "✅ Captured: [${TYPE}] ${SUMMARY} (tool: ${TOOL_NAME})"
