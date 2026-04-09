# MVD Claude

**Persistent memory for AI coding agents — powered by a single portable file.**

Give Claude Code photographic memory across sessions. Every decision, discovery, bug fix, and architectural choice is captured in one `.mv2` file that you can version control, share, and transfer.

## How It Works

```
your-project/
├── CLAUDE.md          # Project instructions (copy from this repo)
├── .claude/commands/  # Slash commands (copy from this repo)
├── scripts/           # Helper scripts (copy from this repo)
└── mvd/
    └── mvd.mv2        # Your agent's brain (local fallback)
```

No database. No cloud. No API keys. No npm. Just one file and the `mvd` binary.

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
- Auto-captured during coding sessions via CLAUDE.md instructions
- Searchable anytime via slash commands

**Why one file?**
- `git commit` → version control your agent's brain
- `scp` → transfer anywhere
- Send to a teammate → instant onboarding

## Installation

### Prerequisites

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- The `mvd` binary in your `$PATH` ([get it from memvid](https://github.com/memvid/memvid))

### Per-Project Setup

Copy the files into your project:

```bash
# Copy CLAUDE.md (or merge into existing)
cp mvd-claude/CLAUDE.md /path/to/your-project/

# Copy slash commands
cp -r mvd-claude/.claude /path/to/your-project/

# Copy helper scripts
cp -r mvd-claude/scripts /path/to/your-project/
```

If you already have a `CLAUDE.md`, append the contents:

```bash
cat mvd-claude/CLAUDE.md >> /path/to/your-project/CLAUDE.md
```

Start a Claude Code session. The agent will automatically:
1. Create `./mvd/mvd.mv2` if it doesn't exist (or use `~/mvd.mv2` if present)
2. Load recent memories as context
3. Capture observations as you work
4. Generate a session summary before ending

Done.

### Global Setup (All Projects — macOS)

To enable memory for every Claude Code project without cloning per-repo:

```bash
# 1. Append memory instructions to your global CLAUDE.md
mkdir -p ~/.claude
cat mvd-claude/CLAUDE.md >> ~/.claude/CLAUDE.md

# 2. Copy slash commands globally
cp -r mvd-claude/.claude/commands ~/.claude/commands

# 3. Copy scripts to a global location
sudo mkdir -p /usr/local/share/mvd
sudo cp mvd-claude/scripts/*.sh /usr/local/share/mvd/
sudo chmod +x /usr/local/share/mvd/*.sh
```

Then update the script paths in `~/.claude/CLAUDE.md`:
- Replace `./scripts/mvd-resolve.sh` → `/usr/local/share/mvd/mvd-resolve.sh`
- Replace `./scripts/mvd-ensure.sh` → `/usr/local/share/mvd/mvd-ensure.sh`

**Or** symlink scripts into each project:
```bash
ln -s /usr/local/share/mvd scripts
```

> **Global config paths on macOS:**
> - `~/.claude/CLAUDE.md` — Global instructions (loaded in every session)
> - `~/.claude/commands/` — Global slash commands
> - `~/.claude/settings.json` — Global settings (hooks, models, etc.)

## Commands

In Claude Code:

```bash
/mvd-stats                          # memory statistics
/mvd-search "authentication"        # find past context
/mvd-ask "why did we choose X?"     # ask your memory
/mvd-recent                         # what happened lately
/mvd-recent 50                      # show more history
/mvd-remember "decided to use Y"    # manually store a memory
/mvd-session-summary                # generate session summary
```

Or just ask naturally — the CLAUDE.md instructions enable the agent to use the memory system autonomously when relevant.

## Architecture

### CLAUDE.md → Automatic Behaviors

The `CLAUDE.md` file is always loaded by Claude Code and instructs the agent to:

| Behavior | When | What |
|---|---|---|
| **Load context** | Conversation start | Runs `mvd timeline` and `mvd stats` to hydrate context |
| **Capture observations** | After significant work | Stores compressed observations via `mvd put` |
| **Session summary** | Before ending | Captures git diff + summary via `mvd put` |

### Slash Commands → Explicit Actions

| Command | Action | MVD Command |
|---|---|---|
| `/mvd-stats` | View statistics | `mvd stats ./mvd/mvd.mv2 --json` |
| `/mvd-search <query>` | Search memories | `mvd find ./mvd/mvd.mv2 --query "<query>" --json` |
| `/mvd-ask <question>` | Ask past context | `mvd ask ./mvd/mvd.mv2 --question "<q>" --context-only --json` |
| `/mvd-recent [n]` | View timeline | `mvd timeline ./mvd/mvd.mv2 --limit <n> --reverse --json` |
| `/mvd-remember <what>` | Store a memory | `mvd put ./mvd/mvd.mv2 --title "..." --label "..." --tag "..."` |
| `/mvd-session-summary` | End-of-session capture | Git diff + summary stored via `mvd put` |

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
mvd-claude/
├── CLAUDE.md                           # Always-active memory system instructions
├── .claude/
│   └── commands/
│       ├── mvd-stats.md                # /mvd-stats
│       ├── mvd-search.md              # /mvd-search
│       ├── mvd-ask.md                 # /mvd-ask
│       ├── mvd-recent.md             # /mvd-recent
│       ├── mvd-remember.md           # /mvd-remember
│       └── mvd-session-summary.md    # /mvd-session-summary
├── scripts/
│   ├── mvd-resolve.sh                 # Resolves memory file path (global/local)
│   ├── mvd-ensure.sh                  # Ensures .mv2 file exists
│   ├── mvd-put.sh                     # Convenience put wrapper
│   └── mvd-capture.sh                 # Auto-classifying observation capture
└── README.md
```

## FAQ

<details>
<summary><b>How is this different from the claude-brain plugin?</b></summary>

This uses the `mvd` binary directly — no Node.js, no npm, no `@memvid/sdk`. Just shell scripts and the compiled Rust binary. It's simpler, faster, and has zero dependencies beyond `mvd` in your PATH.

</details>

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
