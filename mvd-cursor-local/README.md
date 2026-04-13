# MVD Cursor

**Persistent memory for AI coding agents — powered by a single portable file.**

Give your Cursor agent photographic memory across sessions. Every decision, discovery, bug fix, and architectural choice is captured in one `.mv2` file that you can version control, share, and transfer.

## How It Works

```
your-project/
├── .cursor/rules/   # Cursor rules (copy from this repo)
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
- Searchable anytime

**Why one file?**
- `git commit` → version control your agent's brain
- `scp` → transfer anywhere
- Send to a teammate → instant onboarding

## Installation

### Prerequisites

- [Cursor](https://cursor.com) installed
- The `mvd` binary in your `$PATH` ([get it from memvid](https://github.com/memvid/memvid))

### Per-Project Setup

Copy the files into your project:

```bash
cp -r mvd-cursor/.cursor /path/to/your-project/
cp -r mvd-cursor/scripts /path/to/your-project/
```

Open the project in Cursor. The agent will automatically:
1. Create `./mvd/mvd.mv2` if it doesn't exist (or use `~/mvd.mv2` if present)
2. Load recent memories as context
3. Capture observations as you work
4. Generate a session summary before ending

Done.

### Global Setup (All Projects — macOS)

Cursor does **not** support a global filesystem directory for `.mdc` rule files. Instead:

1. **Global rules via Settings UI:**
   - Open Cursor → **Settings** (`Cmd + ,`) → **General** → **Rules for AI**
   - Paste the contents of `mvd-cursor/.cursor/rules/mvd-memory.mdc` (without the frontmatter) into the text field
   - These rules apply to every project automatically

2. **Scripts** still need to be in each project (or globally symlinked):
   ```bash
   # Copy scripts to a global location
   sudo mkdir -p /usr/local/share/mvd
   sudo cp mvd-cursor/scripts/*.sh /usr/local/share/mvd/
   sudo chmod +x /usr/local/share/mvd/*.sh

   # Then in each project, symlink:
   ln -s /usr/local/share/mvd scripts
   ```

3. **Agent-requested rules** (stats, search, ask, etc.) must remain per-project in `.cursor/rules/`. Copy them once per project or symlink the directory.

> **Global config on macOS:**
> - **Cursor Settings** → General → Rules for AI (plain text, no `.mdc` frontmatter)
> - Per-project: `.cursor/rules/*.mdc` (supports frontmatter, globs, alwaysApply)

## How It Works — Rules

Cursor uses `.cursor/rules/*.mdc` files to configure agent behavior. This project provides:

### Always-Active Rule

**`mvd-memory.mdc`** (`alwaysApply: true`) — The core memory system. This is always loaded and instructs the agent to:
- Load context from memory at conversation start
- Proactively capture observations after significant work
- Generate session summaries before ending

### Agent-Requested Rules

These are activated by the agent when relevant based on their descriptions:

| Rule | When It Activates | What It Does |
|---|---|---|
| `mvd-stats.mdc` | User asks about memory stats | Runs `mvd stats` and presents results |
| `mvd-search.mdc` | User wants to search memories | Runs `mvd find` with the query |
| `mvd-ask.mdc` | User asks about past work | Runs `mvd ask` for retrieval + synthesis |
| `mvd-recent.mdc` | User wants recent activity | Runs `mvd timeline` in reverse |
| `mvd-remember.mdc` | User wants to save something | Stores a memory via `mvd put` |
| `mvd-session-summary.mdc` | User wrapping up | Captures git diff + session summary |

### How to Trigger

Just ask naturally:
- *"What do you remember about the auth system?"* → triggers `mvd-ask`
- *"Search my memory for CORS"* → triggers `mvd-search`
- *"Show me recent activity"* → triggers `mvd-recent`
- *"Remember that we chose PostgreSQL"* → triggers `mvd-remember`
- *"Show memory stats"* → triggers `mvd-stats`
- *"Generate a session summary"* → triggers `mvd-session-summary`

## Helper Scripts

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
mvd-cursor/
├── .cursor/
│   └── rules/
│       ├── mvd-memory.mdc              # Always-active memory system (core)
│       ├── mvd-stats.mdc               # Agent-requested: memory statistics
│       ├── mvd-search.mdc              # Agent-requested: search memories
│       ├── mvd-ask.mdc                 # Agent-requested: ask questions
│       ├── mvd-recent.mdc              # Agent-requested: recent timeline
│       ├── mvd-remember.mdc            # Agent-requested: store a memory
│       └── mvd-session-summary.mdc     # Agent-requested: session summary
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
<summary><b>How is this different from Cursor's built-in memory?</b></summary>

Cursor's built-in notepads are UI-based and not portable. MVD memory is:
- **File-based** — lives in your repo, not in Cursor's app data
- **Git-friendly** — commit and version control your agent's knowledge
- **Transferable** — share the `.mv2` file with teammates
- **Searchable** — full-text and semantic search over all memories
- **Persistent** — survives across Cursor updates and reinstalls

</details>

---

Built on **[memvid](https://github.com/memvid/memvid)** — the single-file memory engine for AI agents.
