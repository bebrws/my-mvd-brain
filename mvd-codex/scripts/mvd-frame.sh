#!/usr/bin/env bash
# mvd-frame.sh - View a specific memory frame.
#
# Usage: ./scripts/mvd-frame.sh <frame_id>

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: mvd-frame.sh <frame_id>" >&2
    exit 1
fi

SCRIPT_DIR="$(dirname "$0")"
MEMORY_FILE="$(bash "${SCRIPT_DIR}/mvd-resolve.sh")"

mvd view "${MEMORY_FILE}" "$1"
