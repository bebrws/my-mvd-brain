#!/usr/bin/env bash
# install.sh — install MVD memory rules + hook into ~/.claude.
#
# What this does:
#   1. Strips any existing "## MVD Memory System" section from
#      ~/.claude/CLAUDE.md (preserving any non-MVD content) and appends
#      the fresh CLAUDE.md from this directory.
#   2. Merges the SessionStart hook from settings.snippet.json into
#      ~/.claude/settings.json, removing any previous mvd-related hook.
#
# Idempotent — safe to run repeatedly.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$HOME/.claude"
mkdir -p "$CLAUDE_DIR"

if ! command -v python3 >/dev/null 2>&1; then
  echo "Error: python3 is required (used for safe JSON / Markdown merging)." >&2
  exit 1
fi

# ── 1. CLAUDE.md ──────────────────────────────────────────────────────
python3 - "$ROOT/CLAUDE.md" "$CLAUDE_DIR/CLAUDE.md" <<'PY'
import re
import sys
from pathlib import Path

src = Path(sys.argv[1])
dst = Path(sys.argv[2])
new_block = src.read_text(encoding="utf-8").strip() + "\n"

if dst.exists():
    existing = dst.read_text(encoding="utf-8")

    # Strip any prior MVD section. Matches headings like:
    #   # MVD Memory System (...)
    #   ## MVD Memory System
    #   ### MVD Memory System
    # and consumes lines until the next heading at the SAME OR HIGHER level
    # (i.e. fewer-or-equal #'s) that does NOT start with "MVD".
    # Falls back to consuming to EOF if no such heading exists.
    pattern = re.compile(
        r"^(#{1,3})\s+MVD\s+Memory\s+System.*?"
        r"(?=^\1\s+(?!MVD\s+Memory\s+System)|\Z)",
        re.MULTILINE | re.DOTALL | re.IGNORECASE,
    )
    cleaned = pattern.sub("", existing).rstrip()

    # Also drop any first-line "# CLAUDE.md" placeholder if the rest is empty.
    if cleaned.strip() == "# CLAUDE.md":
        cleaned = ""

    if cleaned:
        merged = cleaned + "\n\n" + new_block
    else:
        merged = new_block
else:
    merged = new_block

dst.write_text(merged, encoding="utf-8")
print(f"Wrote {dst}")
PY

# ── 2. settings.json ─────────────────────────────────────────────────
python3 - "$ROOT/settings.snippet.json" "$CLAUDE_DIR/settings.json" <<'PY'
import json
import sys
from pathlib import Path

snippet = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
dst = Path(sys.argv[2])

if dst.exists():
    existing = json.loads(dst.read_text(encoding="utf-8"))
else:
    existing = {}

existing.setdefault("hooks", {})

def is_mvd_hook_group(group):
    """A hook group whose commands invoke `mvd ` is considered ours."""
    for h in group.get("hooks", []):
        cmd = h.get("command", "")
        if isinstance(cmd, str) and ("mvd " in cmd or "mvd\t" in cmd):
            return True
    return False

for event, groups in snippet.get("hooks", {}).items():
    bucket = existing["hooks"].setdefault(event, [])
    # Remove any prior mvd-installed groups for this event.
    bucket[:] = [g for g in bucket if not is_mvd_hook_group(g)]
    # Append the new groups.
    bucket.extend(groups)
    # Clean up empty event arrays.
    if not bucket:
        del existing["hooks"][event]

if not existing["hooks"]:
    del existing["hooks"]

dst.write_text(json.dumps(existing, indent=2) + "\n", encoding="utf-8")
print(f"Wrote {dst}")
PY

echo ""
echo "Installed. The SessionStart hook activates on the next Claude Code session."
echo "(Open /hooks in Claude Code to reload immediately, or restart.)"
