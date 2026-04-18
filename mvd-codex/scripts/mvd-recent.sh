#!/usr/bin/env bash
# mvd-recent.sh - Show recent memory frames.
#
# Usage: ./scripts/mvd-recent.sh [limit]

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"
LIMIT="${1:-20}"

mvd timeline "${MEMORY_FILE}" --limit "${LIMIT}" --reverse --json
