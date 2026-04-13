#!/usr/bin/env bash
# Install MVD Codex integration into ~/.codex (instructions, hooks, scripts).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEX_DIR="$HOME/.codex"
SCRIPT_DIR="$CODEX_DIR/scripts/mvd"

mkdir -p "$SCRIPT_DIR"

cp "$ROOT/scripts/"*.sh "$SCRIPT_DIR/"
chmod +x "$SCRIPT_DIR/"*.sh

TMP_AGENTS="$(mktemp)"
python3 - "$ROOT/AGENTS.md" "$TMP_AGENTS" "$SCRIPT_DIR" <<'PY'
import re
import sys
from pathlib import Path

src = Path(sys.argv[1])
dest = Path(sys.argv[2])
script_dir = sys.argv[3]
text = src.read_text(encoding="utf-8")
text = re.sub(
    r"bash \./scripts/([A-Za-z0-9_.-]+\.sh)",
    lambda match: f'bash "{script_dir}/{match.group(1)}"',
    text,
)
dest.write_text(text, encoding="utf-8")
PY

if [ -f "$CODEX_DIR/AGENTS.md" ]; then
  {
    printf '\n\n'
    cat "$TMP_AGENTS"
  } >> "$CODEX_DIR/AGENTS.md"
else
  cp "$TMP_AGENTS" "$CODEX_DIR/AGENTS.md"
fi
rm -f "$TMP_AGENTS"

python3 - "$ROOT/hooks.json" "$CODEX_DIR/hooks.json" "$SCRIPT_DIR" <<'PY'
import json
import sys
from pathlib import Path

src = Path(sys.argv[1])
dest = Path(sys.argv[2])
script_dir = sys.argv[3]
data = json.loads(src.read_text(encoding="utf-8"))

for groups in data.get("hooks", {}).values():
    for group in groups:
        for handler in group.get("hooks", []):
            command = handler.get("command")
            if command:
                handler["command"] = command.replace(
                    "bash ./scripts/mvd-codex-hook-handler.sh",
                    f'bash "{script_dir}/mvd-codex-hook-handler.sh"',
                )

dest.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
print(f"Wrote {dest}")
PY

echo ""
echo "Done."
echo "Enable hooks with: codex --enable codex_hooks"
echo "Or merge $ROOT/config.toml.example into ~/.codex/config.toml."
