# MVD Claude Code Integration

Persistent memory for Claude Code, powered by the [`mvd`](https://github.com/memvid/memvid)
single-file memory engine.

Two artifacts ship from this directory:

| File | Lands at | What it does |
|---|---|---|
| `CLAUDE.md` | `~/.claude/CLAUDE.md` | Self-contained global instructions Claude Code loads into every session — auto-stamped writes, scope-aware reads, and `mvd setup` / Gemma cache notes. |
| `settings.snippet.json` | merged into `~/.claude/settings.json` | One `SessionStart` hook that runs `mvd timeline` on `startup|resume|clear` and pipes the result into the model's context. |

No helper scripts, no per-project setup. The CLAUDE.md content is fully
self-contained — Claude Code only needs `mvd` on `$PATH`.

## Prerequisites

- Claude Code installed.
- The `mvd` binary on your `$PATH`. From this repo:
  ```bash
  cargo build --release --bin mvd --features 'cli vec temporal_track local-llm replay'
  cp target/release/mvd ~/bin/mvd          # or anywhere on PATH
  ```
- Optional but recommended: `mvd create ~/mvd.mv2` once, so all projects share
  one capsule. Without it, mvd falls back to per-project `./mvd/mvd.mv2`.

## Install

```bash
cd /path/to/memvid/mvd-claude
./install.sh
```

The installer is idempotent. It will:

1. **Update `~/.claude/CLAUDE.md`** — strips any existing
   `## MVD Memory System` section (preserving any non-MVD content you have)
   and appends the fresh content from this directory's `CLAUDE.md`.
2. **Update `~/.claude/settings.json`** — JSON-merges the SessionStart hook
   from `settings.snippet.json`, removing any previously-installed
   `mvd`-tagged hook before adding the new one. Existing keys (`model`,
   `effortLevel`, your own custom hooks, etc.) are preserved verbatim.

After install, restart Claude Code or open the `/hooks` menu to reload —
Claude Code only re-reads `settings.json` at session start.

## Manual install

If you'd rather merge by hand:

1. Copy the body of `CLAUDE.md` into `~/.claude/CLAUDE.md`, replacing any
   prior MVD section.
2. Open `~/.claude/settings.json` and add the `hooks` block from
   `settings.snippet.json`. If you already have a `hooks` key, deep-merge
   `SessionStart` into it.

The hook command itself, copy-paste:

```bash
MVD_FILE="$HOME/mvd.mv2"; [ -f "$MVD_FILE" ] || MVD_FILE="./mvd/mvd.mv2"; if command -v mvd >/dev/null 2>&1 && [ -f "$MVD_FILE" ]; then mvd timeline "$MVD_FILE" --limit 15 --reverse --all-repos --json 2>/dev/null; fi; exit 0
```

## What gets sent to the model

- **At session start**: the hook runs the inline command above. Its stdout (a
  JSON array of recent timeline entries, or `[]` if the capsule is empty / not
  yet created) is injected into the model's context as system context.
- **During a session**: the model follows the instructions in `CLAUDE.md` — it
  captures substantive work via `mvd put` (auto-tagged with current repo /
  branch / `harness=claude-code`), recalls via `mvd find` / `mvd vec` /
  `mvd ask`, and writes a session-summary frame at end-of-session.

## Uninstall

```bash
# Remove the MVD section from CLAUDE.md.
python3 -c "
import re, pathlib
p = pathlib.Path.home() / '.claude' / 'CLAUDE.md'
text = p.read_text()
text = re.sub(
    r'^(#{1,3})\s+MVD\s+Memory\s+System.*?(?=^\1\s+(?!MVD\s+Memory\s+System)|\Z)',
    '', text, flags=re.MULTILINE | re.DOTALL | re.IGNORECASE,
)
p.write_text(text.rstrip() + '\n' if text.strip() else '')"

# Remove the MVD hook from settings.json.
python3 -c "
import json, pathlib
p = pathlib.Path.home() / '.claude' / 'settings.json'
data = json.loads(p.read_text())
hooks = data.get('hooks', {})
for event in list(hooks):
    hooks[event] = [g for g in hooks[event]
        if not any('mvd ' in h.get('command', '') for h in g.get('hooks', []))]
    if not hooks[event]:
        del hooks[event]
if not hooks: data.pop('hooks', None)
p.write_text(json.dumps(data, indent=2) + '\n')"
```

## Troubleshooting

**The hook doesn't fire.** Open `/hooks` in Claude Code or restart — settings
are read once at session start.

**`mvd timeline` returns `[]` even though the capsule has frames.** That's a
known mvd-side issue with default-scoped timeline queries; the hook still
exits 0 cleanly. Worst case the model gets empty context — the in-CLAUDE.md
instructions still work, and you can ask the model to query memory directly.

**The model isn't auto-capturing.** It's instruction-driven, not hook-driven —
look at section 3 of the installed `~/.claude/CLAUDE.md`. Old `mvd` binaries
won't auto-stamp `repo`/`branch`/`harness`; rebuild from this repo if you
last installed before that change.

**You see "Loading local LLM (...) Quantizing in-place..." every time.**
Run `mvd setup` once to pre-quantize Gemma to UQFF (~1–2 min once); future
loads drop to ~5 s. Section 7 of the installed `CLAUDE.md` covers this.

## File layout

```
mvd-claude/
├── README.md               # this file
├── CLAUDE.md               # global Claude Code instructions
├── settings.snippet.json   # SessionStart hook to merge into ~/.claude/settings.json
└── install.sh              # idempotent installer (uses python3 for safe merging)
```
