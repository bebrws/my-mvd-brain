#!/usr/bin/env bash
# mvd-stats.sh - Show memory file statistics.
#
# Usage: ./scripts/mvd-stats.sh

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"

mvd stats "${MEMORY_FILE}" --json
