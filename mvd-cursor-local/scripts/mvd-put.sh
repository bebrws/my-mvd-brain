#!/usr/bin/env bash
# mvd-put.sh — Store content into the memory file
#
# Usage: echo "content" | bash mvd-put.sh "title" "label" "tag"
#    or: bash mvd-put.sh "title" "label" "tag" <<< "content"
#    or: bash mvd-put.sh "title" "label" "tag" --input file.txt
#
# Arguments:
#   $1 — Title/summary (required)
#   $2 — Label/type: discovery, decision, problem, solution, pattern,
#         warning, success, refactor, bugfix, feature, session (required)
#   $3 — Tag/tool source (optional, defaults to "agent")
#   --input FILE — Read content from file instead of stdin

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"

# Check args
if [ $# -lt 2 ]; then
    echo "Usage: echo 'content' | mvd-put.sh <title> <label> [tag]" >&2
    echo "   or: mvd-put.sh <title> <label> [tag] --input <file>" >&2
    exit 1
fi

TITLE="$1"
LABEL="$2"
TAG="${3:-agent}"

# Ensure memory file exists
if [ ! -f "${MEMORY_FILE}" ]; then
    bash "${SCRIPT_DIR}/mvd-ensure.sh"
    MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
fi

# Build the mvd put command
CMD=(mvd put "${MEMORY_FILE}" --title "${TITLE}" --label "${LABEL}" --tag "${TAG}")

# Check for --input flag
if [ "${4:-}" = "--input" ] && [ -n "${5:-}" ]; then
    CMD+=(--input "${5}")
    "${CMD[@]}"
else
    # Read from stdin
    "${CMD[@]}"
fi

echo "✅ Stored: [${LABEL}] ${TITLE}"
