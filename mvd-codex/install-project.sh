#!/usr/bin/env bash
# Copy Codex memory instructions, hooks, and scripts into a project root.
set -euo pipefail

if [ "${1:-}" = "" ] || [ ! -d "$1" ]; then
  echo "Usage: $0 /path/to/your/project" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="$(cd "$1" && pwd)"

if [ -f "$DEST/AGENTS.md" ]; then
  {
    printf '\n\n'
    cat "$ROOT/AGENTS.md"
  } >> "$DEST/AGENTS.md"
else
  cp "$ROOT/AGENTS.md" "$DEST/AGENTS.md"
fi

cp "$ROOT/hooks.json" "$DEST/hooks.json"
cp -R "$ROOT/scripts" "$DEST/"
chmod +x "$DEST/scripts/"*.sh

echo "Installed into $DEST"
echo "  $DEST/AGENTS.md"
echo "  $DEST/hooks.json"
echo "  $DEST/scripts/"
echo ""
echo "Enable Codex hooks with: codex --enable codex_hooks"
echo "Or merge mvd-codex/config.toml.example into ~/.codex/config.toml."
