#!/usr/bin/env bash
# mvd-resolve-global.sh — Resolve the memory file path from any working directory
#
# Unlike mvd-resolve.sh (which uses relative paths), this script uses absolute
# paths so it works correctly when called from hook scripts that may run from
# ~/.cursor/ or any other directory.
#
# Priority:
#   1. $HOME/mvd.mv2 (global, if it exists)
#   2. $CURSOR_PROJECT_DIR/mvd/mvd.mv2 (per-project, using Cursor's env var)
#   3. Falls back to $HOME/mvd.mv2 if no project dir is known
#
# Usage: MVD_FILE=$(bash /path/to/mvd-resolve-global.sh)

if [ -f "$HOME/mvd.mv2" ]; then
    echo "$HOME/mvd.mv2"
elif [ -n "$CURSOR_PROJECT_DIR" ] && [ -f "$CURSOR_PROJECT_DIR/mvd/mvd.mv2" ]; then
    echo "$CURSOR_PROJECT_DIR/mvd/mvd.mv2"
else
    # Default to global — hooks will create it if needed
    echo "$HOME/mvd.mv2"
fi
