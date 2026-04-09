# MVD AntiGravity

**Persistent memory for AI coding agents — powered by a single portable file.**

Give your AntiGravity agent photographic memory across sessions. Every decision, discovery, bug fix, and architectural choice is captured in one `.mv2` file that you can version control, share, and transfer.

## How It Works

```
your-project/
├── _agent/          # AntiGravity config (copy from this repo)
├── scripts/         # Helper scripts (copy from this repo)
└── mvd/
    └── mvd.mv2      # Your agent's brain (local fallback)
```

No database. No cloud. No API keys. Just one file.

### Memory File Location

The system checks for memory files in this order:
1. **Global**: `$HOME/mvd.mv2` — shared across all projects
2. **Local**: `./mvd/mvd.mv2` — per-project, created automatically if no global file exists

To use a single global memory across all your projects, create one:
```bash
mvd create ~/mvd.mv2
```

**What gets captured:**
- Session context, decisions, bugs, solutions
- Auto-captured during coding sessions via rules
- Searchable anytime via workflows

**Why one file?**
- `git commit` → version control your agent's brain
- `scp` → transfer anywhere
- Send to a teammate → instant onboarding

## Installation

### Prerequisites

- [AntiGravity](https://antigravity.google) installed
- The `mvd` binary in your `$PATH` ([get it from memvid](https://github.com/memvid/memvid))

### Per-Project Setup

Copy the `_agent/` and `scripts/` directories into your project:

```bash
cp -r mvd-antigravity/_agent /path/to/your-project/
cp -r mvd-antigravity/scripts /path/to/your-project/
```

Start a new AntiGravity conversation. The agent will automatically:
1. Create `./mvd/mvd.mv2` if it doesn't exist (or use `~/mvd.mv2` if present)
2. Load recent memories as context
3. Capture observations as you work
4. Generate a session summary before ending

Done.

### Global Setup (All Projects — macOS)

To enable memory for every AntiGravity project without cloning per-repo:

```bash
# 1. Append memory rules to your global GEMINI.md
cat mvd-antigravity/_agent/rules/mvd-memory.md >> ~/.gemini/GEMINI.md

# 2. Copy scripts to a global location
sudo mkdir -p /usr/local/share/mvd
sudo cp mvd-antigravity/scripts/*.sh /usr/local/share/mvd/
sudo chmod +x /usr/local/share/mvd/*.sh
```

Then update the script paths in `~/.gemini/GEMINI.md`:
- Replace `./scripts/mvd-resolve.sh` → `/usr/local/share/mvd/mvd-resolve.sh`
- Replace `./scripts/mvd-ensure.sh` → `/usr/local/share/mvd/mvd-ensure.sh`

**Or** symlink scripts into each project:
```bash
ln -s /usr/local/share/mvd scripts
```

> **Global config paths on macOS:**
> - `~/.gemini/GEMINI.md` — Global rules (loaded in every session)
> - `~/.gemini/AGENTS.md` — Cross-tool global rules standard (also supported)

## Workflows

Run these as slash commands in AntiGravity:

```
/mvd-stats                        # memory statistics
/mvd-search "authentication"      # find past context
/mvd-ask "why did we choose X?"   # ask your memory
/mvd-recent                       # what happened lately
/mvd-remember "decided to use Y"  # manually store a memory
/mvd-session-summary              # generate session summary
```

Or just ask naturally — the agent's skill definition enables it to use the memory system autonomously when relevant.

## Architecture

### Rules → Automatic Behaviors

The rules file (`_agent/rules/mvd-memory.md`) instructs the agent to:

| Behavior | When | What |
|---|---|---|
| **Load context** | Conversation start | Runs `mvd timeline` and `mvd stats` to hydrate context |
| **Capture observations** | After significant work | Stores compressed observations via `mvd put` |
| **Session summary** | Before ending | Captures git diff + summary via `mvd put` |

### Workflows → Explicit Commands

| Workflow | Maps to | MVD Command |
|---|---|---|
| `/mvd-stats` | View statistics | `mvd stats "$MVD_FILE" --json` |
| `/mvd-search <query>` | Search memories | `mvd find "$MVD_FILE" --query "<query>" --json` |
| `/mvd-ask <question>` | Ask past context | `mvd ask "$MVD_FILE" --question "<q>" --context-only --json` |
| `/mvd-recent [n]` | View timeline | `mvd timeline "$MVD_FILE" --limit <n> --reverse --json` |
| `/mvd-remember` | Store a memory | `mvd put "$MVD_FILE" --title "..." --label "..." --tag "..."` |
| `/mvd-session-summary` | End-of-session capture | Git diff + summary stored via `mvd put` |

> `$MVD_FILE` is resolved by `scripts/mvd-resolve.sh` — global `$HOME/mvd.mv2` if it exists, otherwise local `./mvd/mvd.mv2`.

### Skills → Model-Invoked Memory

The skill (`_agent/skills/mvd-memory/SKILL.md`) lets the agent autonomously decide when to search or store memories based on task context — no explicit command needed.

### Helper Scripts

| Script | Purpose |
|---|---|
| `scripts/mvd-resolve.sh` | Resolves memory file path (`$HOME/mvd.mv2` → `./mvd/mvd.mv2`) |
| `scripts/mvd-ensure.sh` | Creates the memory file if it doesn't exist |
| `scripts/mvd-put.sh` | Convenience wrapper for `mvd put` with stdin support |
| `scripts/mvd-capture.sh` | Auto-classifies observations by type (discovery, bugfix, feature, etc.) |

## Memory Types

Observations are classified into these types:

| Type | Description |
|---|---|
| `discovery` | New information discovered |
| `decision` | Important decision made |
| `problem` | Problem or error encountered |
| `solution` | Solution implemented |
| `pattern` | Pattern recognized |
| `warning` | Warning or concern noted |
| `success` | Successful outcome |
| `refactor` | Code refactoring done |
| `bugfix` | Bug fixed |
| `feature` | Feature added |
| `session` | Session summary |

## File Structure

```
mvd-antigravity/
├── _agent/
│   ├── rules/
│   │   └── mvd-memory.md                # Always-active memory system rules
│   ├── workflows/
│   │   ├── mvd-stats.md                 # /mvd-stats
│   │   ├── mvd-search.md               # /mvd-search
│   │   ├── mvd-ask.md                  # /mvd-ask
│   │   ├── mvd-recent.md               # /mvd-recent
│   │   ├── mvd-remember.md             # /mvd-remember
│   │   └── mvd-session-summary.md      # /mvd-session-summary
│   └── skills/
│       └── mvd-memory/
│           └── SKILL.md                 # Model-invoked memory skill
├── scripts/
│   ├── mvd-resolve.sh                   # Resolves memory file path (global/local)
│   ├── mvd-ensure.sh                    # Ensures .mv2 file exists
│   ├── mvd-put.sh                       # Convenience put wrapper
│   └── mvd-capture.sh                   # Auto-classifying observation capture
└── README.md
```

## FAQ

<details>
<summary><b>How big is the memory file?</b></summary>

Empty: ~70KB. Grows ~1KB per memory. A year of daily use stays well under 10MB.

</details>

<details>
<summary><b>Is it private?</b></summary>

100% local. Nothing leaves your machine. The `.mv2` file is just a file on disk.

</details>

<details>
<summary><b>How fast?</b></summary>

Sub-millisecond search. Native Rust core. Searches 10K+ memories in <1ms.

</details>

<details>
<summary><b>Reset memory?</b></summary>

```bash
rm -rf ./mvd/
```

</details>

<details>
<summary><b>Can I encrypt it?</b></summary>

Yes. Use `mvd lock` to create an encrypted capsule (`.mv2e`) and `mvd unlock` to decrypt.

</details>

---

Built on **[memvid](https://github.com/memvid/memvid)** — the single-file memory engine for AI agents.
