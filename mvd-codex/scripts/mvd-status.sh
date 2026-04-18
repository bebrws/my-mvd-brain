#!/usr/bin/env bash
# mvd-status.sh - Print the active memory file, recent timeline, and stats.
#
# Usage: ./scripts/mvd-status.sh [timeline_limit]

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
LIMIT="${1:-10}"

printf '%s\n' "${MEMORY_FILE}"
mvd timeline "${MEMORY_FILE}" --limit "${LIMIT}" --reverse --json
mvd stats "${MEMORY_FILE}" --json
