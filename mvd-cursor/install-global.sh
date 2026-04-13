#!/usr/bin/env bash
# Install MVD Cursor integration into ~/.cursor (rules, hooks, helper scripts).
# Paths in rules point at $HOME/.cursor/scripts/mvd/*.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SH="$HOME/.cursor/scripts/mvd"
HOOKS="$HOME/.cursor/hooks"
RULES="$HOME/.cursor/rules"

mkdir -p "$SH" "$HOOKS" "$RULES"

cp "$ROOT/scripts/"*.sh "$SH/"
chmod +x "$SH/"*.sh

cp "$ROOT/.cursor/hooks/mvd-hook-handler.sh" "$ROOT/.cursor/hooks/mvd-resolve-global.sh" "$HOOKS/"
chmod +x "$HOOKS/mvd-hook-handler.sh" "$HOOKS/mvd-resolve-global.sh"

cp "$ROOT/global-user/hooks.json" "$HOME/.cursor/hooks.json"

export MVD_CURSOR_ROOT="$ROOT"
python3 <<'PY'
import os
import re
from pathlib import Path

home = str(Path.home())
root = Path(os.environ["MVD_CURSOR_ROOT"])
out_rules = Path.home() / ".cursor" / "rules"
pat = re.compile(r"bash ./scripts/([A-Za-z0-9_.-]+\.sh)")

for path in (root / ".cursor" / "rules").glob("*.mdc"):
    text = path.read_text(encoding="utf-8")
    text = pat.sub(lambda m: f'bash "{home}/.cursor/scripts/mvd/{m.group(1)}"', text)
    (out_rules / path.name).write_text(text, encoding="utf-8")
    print(f"Wrote {out_rules / path.name}")
PY

echo ""
echo "Done. Restart Cursor so hooks reload."
echo "Optional: mvd create ~/mvd.mv2   # shared memory for all projects"
