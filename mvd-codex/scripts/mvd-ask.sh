#!/usr/bin/env bash
# mvd-ask.sh - Retrieve memory context for a natural-language question.
#
# Usage: ./scripts/mvd-ask.sh <question> [top_k]

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: mvd-ask.sh <question> [top_k]" >&2
    exit 1
fi

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
QUESTION="$1"
TOP_K="${2:-8}"

mvd ask "${MEMORY_FILE}" --question "${QUESTION}" --context-only --top-k "${TOP_K}" --json
