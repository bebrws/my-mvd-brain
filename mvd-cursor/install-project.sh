#!/usr/bin/env bash
# Copy .cursor (rules + hooks) and scripts into a project repository root.
set -euo pipefail

if [ "${1:-}" = "" ] || [ ! -d "$1" ]; then
  echo "Usage: $0 /path/to/your/project" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="$(cd "$1" && pwd)"

cp -R "$ROOT/.cursor" "$DEST/"
cp -R "$ROOT/scripts" "$DEST/"
chmod +x "$DEST/.cursor/hooks/"*.sh 2>/dev/null || true
chmod +x "$DEST/scripts/"*.sh

echo "Installed into $DEST"
echo "  $DEST/.cursor/   (rules + hooks + hooks.json)"
echo "  $DEST/scripts/"
echo "Open the folder in Cursor. Optional: mvd create \"\$DEST/mvd/mvd.mv2\" for project-local memory."
