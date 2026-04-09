# MVD Codex

**Persistent memory for AI coding agents — powered by a single portable file.**

Give your OpenAI Codex CLI agent photographic memory across sessions. Every decision, discovery, bug fix, and architectural choice is captured in one `.mv2` file that you can version control, share, and transfer.

## How It Works

```
your-project/
├── AGENTS.md        # Codex instructions (copy from this repo)
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
- Auto-captured during coding sessions via AGENTS.md instructions
- Searchable anytime

**Why one file?**
- `git commit` → version control your agent's brain
- `scp` → transfer anywhere
- Send to a teammate → instant onboarding

## Installation

### Prerequisites

- [OpenAI Codex CLI](https://github.com/openai/codex) installed
- The `mvd` binary in your `$PATH` ([get it from memvid](https://github.com/memvid/memvid))

### Per-Project Setup

Copy the files into your project:

```bash
cp mvd-codex/AGENTS.md /path/to/your-project/
cp -r mvd-codex/scripts /path/to/your-project/
```

If you already have an `AGENTS.md`, append the contents:

```bash
cat mvd-codex/AGENTS.md >> /path/to/your-project/AGENTS.md
```

### Global Setup (All Projects)

Codex reads `~/.codex/AGENTS.md` globally. To enable memory for every project without cloning per-repo:

```bash
# 1. Create the global codex config directory
mkdir -p ~/.codex

# 2. Copy the AGENTS.md (or append to existing)
cp mvd-codex/AGENTS.md ~/.codex/AGENTS.md

# 3. Copy scripts to a global location
sudo mkdir -p /usr/local/share/mvd
sudo cp mvd-codex/scripts/*.sh /usr/local/share/mvd/
sudo chmod +x /usr/local/share/mvd/*.sh
```

Then update the script paths in `~/.codex/AGENTS.md` to use the global location:
- Replace `./scripts/mvd-resolve.sh` with `/usr/local/share/mvd/mvd-resolve.sh`
- Replace `./scripts/mvd-ensure.sh` with `/usr/local/share/mvd/mvd-ensure.sh`

**Or** create symlinks in each project:
```bash
ln -s /usr/local/share/mvd scripts
```

Start a Codex session. The agent will automatically:
1. Create `./mvd/mvd.mv2` if it doesn't exist (or use `~/mvd.mv2` if present)
2. Load recent memories as context
3. Capture observations as you work
4. Generate a session summary before ending

Done.

## Architecture

### AGENTS.md → All Behaviors

Codex uses a single `AGENTS.md` file for all instructions. Unlike other harnesses, there are no separate slash commands or rule files. Everything is in one file:

| Behavior | When | What |
|---|---|---|
| **Load context** | Conversation start | Runs `mvd timeline` and `mvd stats` to hydrate context |
| **Capture observations** | After significant work | Stores compressed observations via `mvd put` |
| **Session summary** | Before ending | Captures git diff + summary via `mvd put` |
| **Search/ask** | When user queries memory | Runs `mvd find` or `mvd ask` |
| **Store** | When user says "remember" | Stores via `mvd put` with classification |

### Memory Commands

Ask the agent naturally — it knows these commands from the AGENTS.md:

- *"Search my memory for authentication"* → `mvd find`
- *"What do you remember about the API design?"* → `mvd ask`
- *"Show recent memories"* → `mvd timeline`
- *"Remember that we chose PostgreSQL"* → `mvd put`
- *"Show memory stats"* → `mvd stats`
- *"Generate a session summary"* → `mvd put` with git diff

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
mvd-codex/
├── AGENTS.md                           # All memory system instructions
├── scripts/
│   ├── mvd-resolve.sh                  # Resolves memory file path (global/local)
│   ├── mvd-ensure.sh                   # Ensures .mv2 file exists
│   ├── mvd-put.sh                      # Convenience put wrapper
│   └── mvd-capture.sh                  # Auto-classifying observation capture
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

<details>
<summary><b>How does Codex discover AGENTS.md?</b></summary>

Codex builds an instruction chain:
1. **Global**: `~/.codex/AGENTS.md`
2. **Repo root**: `./AGENTS.md`
3. **Subdirectory**: walks from root to your current directory

Closer files take precedence. See the [Codex docs](https://github.com/openai/codex) for details.

</details>

---

Built on **[memvid](https://github.com/memvid/memvid)** — the single-file memory engine for AI agents.
