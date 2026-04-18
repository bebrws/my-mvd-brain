#!/usr/bin/env bash
# mvd-search.sh - Search memories with lexical retrieval.
#
# Usage: ./scripts/mvd-search.sh <query> [top_k]

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: mvd-search.sh <query> [top_k]" >&2
    exit 1
fi

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
QUERY="$1"
TOP_K="${2:-10}"

mvd find "${MEMORY_FILE}" --query "${QUERY}" --top-k "${TOP_K}" --json
