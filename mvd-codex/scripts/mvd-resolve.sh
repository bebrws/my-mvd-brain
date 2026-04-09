#!/usr/bin/env bash
# mvd-resolve.sh — Resolve the memory file path
#
# Priority:
#   1. $HOME/mvd.mv2 (global, if it exists)
#   2. ./mvd/mvd.mv2 (local, per-project)
#
# Usage: MVD_FILE=$(bash ./scripts/mvd-resolve.sh)

if [ -f "$HOME/mvd.mv2" ]; then
    echo "$HOME/mvd.mv2"
else
    echo "./mvd/mvd.mv2"
fi
