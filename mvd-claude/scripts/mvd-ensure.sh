#!/usr/bin/env bash
# mvd-ensure.sh — Ensure the memory file exists, create it if not
#
# Replaces smart-install.ts from claude-brain.
# Instead of installing npm dependencies, this just ensures the .mv2 file exists.

set -euo pipefail

MEMORY_DIR="./mvd"
MEMORY_FILE="${MEMORY_DIR}/mvd.mv2"

# Check if mvd binary is available
if ! command -v mvd &>/dev/null; then
    echo "ERROR: 'mvd' binary not found in PATH." >&2
    echo "Please install mvd or add it to your PATH." >&2
    exit 1
fi

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
