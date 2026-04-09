#!/usr/bin/env bash
# mvd-ensure.sh — Ensure the memory file exists, create it if not
#
# Priority:
#   1. $HOME/mvd.mv2 (global) — if it exists, use it; don't create local
#   2. ./mvd/mvd.mv2 (local)  — create if global doesn't exist

set -euo pipefail

# Check if mvd binary is available
if ! command -v mvd &>/dev/null; then
    echo "ERROR: 'mvd' binary not found in PATH." >&2
    echo "Please install mvd or add it to your PATH." >&2
    exit 1
fi

# Check for global memory file first
if [ -f "$HOME/mvd.mv2" ]; then
    echo "✅ Using global memory file: $HOME/mvd.mv2"
    exit 0
fi

# Fall back to local memory file
MEMORY_DIR="./mvd"
MEMORY_FILE="${MEMORY_DIR}/mvd.mv2"

# Create directory if needed
if [ ! -d "${MEMORY_DIR}" ]; then
    mkdir -p "${MEMORY_DIR}"
    echo "Created memory directory: ${MEMORY_DIR}"
fi

# Create memory file if it doesn't exist
if [ ! -f "${MEMORY_FILE}" ]; then
    echo "Creating new memory file: ${MEMORY_FILE}"
    mvd create "${MEMORY_FILE}"
    echo "✅ Memory file created successfully."
else
    echo "✅ Memory file exists: ${MEMORY_FILE}"
fi
