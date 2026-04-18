# MVD Codex

**Persistent memory for AI coding agents — powered by a single portable file.**

Give your OpenAI Codex CLI agent photographic memory across sessions. Every decision, discovery, bug fix, and architectural choice is captured in one `.mv2` file that you can version control, share, and transfer.

## How It Works

```
your-project/
├── AGENTS.md        # Codex instructions (copy from this repo)
├── hooks.json       # Optional Codex hooks for automatic capture
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
- Automatically captured by Codex hooks when `codex_hooks` is enabled
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
cp mvd-codex/hooks.json /path/to/your-project/
cp -r mvd-codex/scripts /path/to/your-project/
chmod +x /path/to/your-project/scripts/*.sh
```

If you already have an `AGENTS.md`, append the contents:

```bash
cat mvd-codex/AGENTS.md >> /path/to/your-project/AGENTS.md
```

Or use the installer:

```bash
cd /path/to/memvid/mvd-codex
./install-project.sh /path/to/your-project
```

### Enable Codex Hooks

Codex hook support is currently behind the `codex_hooks` feature flag in Codex CLI builds that include it. Without this flag, `hooks.json` is ignored and persistence still works through the `AGENTS.md` instructions.

Enable hooks for a launch:

```bash
codex --enable codex_hooks
```

Or merge `mvd-codex/config.toml.example` into `~/.codex/config.toml`:

```toml
[features]
codex_hooks = true
```

### Allow Global Memory in the Sandbox

If you use `$HOME/mvd.mv2`, Codex workspace-write sessions may need approval before reading or writing the memory file because it is outside the current repo. You can avoid repeated prompts in either of these ways:

```bash
codex --add-dir "$HOME"
```

Or add a persistent writable root to `~/.codex/config.toml`:

```toml
sandbox_mode = "workspace-write"

[sandbox_workspace_write]
writable_roots = ["/Users/YOUR_USER"]
```

Use the narrowest parent directory that works for your setup. Codex currently configures additional workspace-write access as directories, so `~/mvd.mv2` itself is usually too narrow.

### Global Setup (All Projects)

Codex reads `~/.codex/AGENTS.md` globally. To enable memory for every project without cloning per-repo:

```bash
mkdir -p ~/.codex
cp mvd-codex/AGENTS.md ~/.codex/AGENTS.md
cp mvd-codex/hooks.json ~/.codex/hooks.json
mkdir -p ~/.codex/scripts/mvd
cp mvd-codex/scripts/*.sh ~/.codex/scripts/mvd/
chmod +x ~/.codex/scripts/mvd/*.sh
```

Then update the script paths in `~/.codex/AGENTS.md` and `~/.codex/hooks.json` to use `~/.codex/scripts/mvd/...`.

The global installer does this path rewrite automatically:

```bash
cd /path/to/memvid/mvd-codex
./install-global.sh
```

Start a Codex session. The agent will automatically:
1. Create `./mvd/mvd.mv2` if it doesn't exist (or use `~/mvd.mv2` if present)
2. Load recent memories as context
3. Capture observations as you work
4. Generate a session summary before ending

Done.

## Architecture

### AGENTS.md + Hooks

Codex uses `AGENTS.md` for model-visible instructions. `hooks.json` is optional and runs outside the model when the `codex_hooks` feature is enabled:

| Behavior | When | What |
|---|---|---|
| **Load context** | Conversation start | Runs `mvd timeline` and `mvd stats` to hydrate context |
| **Hook context** | `session_start` / `user_prompt_submit` | Adds recent and relevant memories as extra context |
| **Hook capture** | `post_tool_use` / `stop` | Stores tool results and lifecycle events through `mvd put` |
| **Agent capture** | After significant work | Stores compressed reasoning, decisions, and outcomes via `mvd put` |
| **Session summary** | Before ending | Captures git diff + summary via `mvd put` |
| **Search/ask** | When user queries memory | Runs `mvd find` or `mvd ask` |
| **Store** | When user says "remember" | Stores via `mvd put` with classification |

### Memory Commands

Ask the agent naturally — it knows these commands from the AGENTS.md:

- *"Search my memory for authentication"* → `scripts/mvd-search.sh`
- *"What do you remember about the API design?"* → `scripts/mvd-ask.sh`
- *"Show recent memories"* → `scripts/mvd-recent.sh`
- *"Remember that we chose PostgreSQL"* → `scripts/mvd-put.sh`
- *"Show memory stats"* → `scripts/mvd-stats.sh`
- *"Show memory status"* → `scripts/mvd-status.sh`
- *"Generate a session summary"* → `scripts/mvd-put.sh` with git diff

### Helper Scripts

| Script | Purpose |
|---|---|
| `scripts/mvd-resolve.sh` | Resolves memory file path (`$HOME/mvd.mv2` → `./mvd/mvd.mv2`) |
| `scripts/mvd-ensure.sh` | Creates the memory file if it doesn't exist |
| `scripts/mvd-put.sh` | Convenience wrapper for `mvd put` with stdin support |
| `scripts/mvd-capture.sh` | Auto-classifies observations by type (discovery, bugfix, feature, etc.) |
| `scripts/mvd-search.sh` | Searches memories with `mvd find` |
| `scripts/mvd-ask.sh` | Retrieves memory context with `mvd ask` |
| `scripts/mvd-recent.sh` | Shows recent timeline frames |
| `scripts/mvd-stats.sh` | Shows memory file statistics |
| `scripts/mvd-status.sh` | Prints active memory file, recent timeline, and stats |
| `scripts/mvd-frame.sh` | Views a specific memory frame |
| `scripts/mvd-codex-hook-handler.sh` | Codex hook handler for session context and automatic capture |

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
├── hooks.json                          # Optional Codex hooks
├── config.toml.example                 # Feature flag snippet for hooks
├── install-project.sh                  # Install into one project
├── install-global.sh                   # Install into ~/.codex
├── scripts/
│   ├── mvd-resolve.sh                  # Resolves memory file path (global/local)
│   ├── mvd-ensure.sh                   # Ensures .mv2 file exists
│   ├── mvd-put.sh                      # Convenience put wrapper
│   ├── mvd-capture.sh                  # Auto-classifying observation capture
│   ├── mvd-search.sh                   # Search wrapper
│   ├── mvd-ask.sh                      # Ask/context wrapper
│   ├── mvd-recent.sh                   # Timeline wrapper
│   ├── mvd-stats.sh                    # Stats wrapper
│   ├── mvd-status.sh                   # Combined status wrapper
│   ├── mvd-frame.sh                    # Frame view wrapper
│   └── mvd-codex-hook-handler.sh       # Codex hook capture/context handler
└── README.md
```

## FAQ

<details>
<summary><b>How big is the memory file?</b></summary>

Empty: ~70KB. Grows ~1KB per memory. A year of daily use stays well under 10MB.

</details>

<details>
<summary><b>Are hooks required?</b></summary>

No. `AGENTS.md` still tells Codex to load, search, and save memories. Hooks add hands-off lifecycle capture for Codex builds that support `codex_hooks`.

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
