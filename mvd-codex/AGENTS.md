# AGENTS.md

## MVD Memory System

You have access to a **persistent memory system** powered by `mvd`, a single-file memory engine.
All observations, decisions, discoveries, and learnings are stored in a portable `.mv2` file.

### Instructions vs Hooks

- `AGENTS.md` instructions tell you when and how to use memory.
- Codex hooks, when installed and enabled, run outside the model and automatically capture session starts, prompts, tool results, and stop events.
- Hooks record the raw lifecycle. You still store concise reasoning-only outcomes when hooks would miss the why behind a decision.

### Mandatory Agent Capture

Before finishing any turn where you did substantive work, store at least one memory when any of these happened:

- You created or changed files that affect behavior, APIs, configuration, docs, or installation.
- You ran commands whose result matters, such as tests, builds, lint, migrations, or debugging probes.
- You made a decision, discovered a bug, resolved a problem, or learned project context that should survive into later sessions.

Prefer the capture helper:

```bash
./scripts/mvd-capture.sh "<tool-name>" "<one-line summary>" <<'EOF'
2-5 sentences, compressed facts only.
EOF
```

If hooks are installed, avoid duplicating raw shell output or file-edit details already captured by hooks. Add only the durable reasoning, decisions, and outcomes.

### Memory File Location

The memory file is resolved with this priority:
1. **Global**: `$HOME/mvd.mv2` — if it exists, always use this
2. **Local**: `./mvd/mvd.mv2` — per-project fallback

To resolve the path, run:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
```

Use `$MVD_FILE` in all subsequent `mvd` commands within the conversation.

### Memory File Setup

At the **beginning of every conversation**, do the following SILENTLY (no need to tell the user):

1. Run the ensure script to create the memory file if it doesn't exist:
   ```bash
   bash ./scripts/mvd-ensure.sh
   ```
2. Resolve the memory file path:
   ```bash
   MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
   ```
3. Load recent context **scoped to the current repo + branch** by running:
   ```bash
   ./scripts/mvd-recent.sh 20 2>/dev/null
   ```
4. Check memory stats:
   ```bash
   ./scripts/mvd-stats.sh 2>/dev/null
   ```

Use the returned context to inform your work. Reference relevant past memories when they apply to the current task.

> **Session-start scope:** the SessionStart hook loads context **scoped to the
> current git repository and branch only**. If the workspace is not a git repo,
> no context is auto-loaded — query memory explicitly in that case. To recall
> across repos during a session, pass `--all-repos` to `mvd find`/`mvd vec`/
> `mvd ask` (see §5).

### Repository / Branch / Harness Scope (auto-applied)

Recent versions of `mvd` automatically tag every new frame with the **current git repository, branch, and agent harness** (cursor / claude-code / codex / aider / ...). All read commands (`find`, `vec`, `ask`, `timeline`, `stats`) default to filtering results to the **current repo + branch**, so the user sees what's relevant to where they are working.

What you need to know:

- **Writes are stamped automatically** — `mvd put` from this directory will tag the frame with this repo and branch and `harness=codex`. No flags needed. Override with `--repo <id>`, `--branch <name>`, or `--no-scope`.
- **Reads filter to the current repo by default.** When the user asks about other repos, broaden the scope:
  - `--repo <id>` — query a specific other repo (e.g. `mvd find ... --repo github.com/owner/other`)
  - `--all-repos` (alias `--global`) — span every repository the capsule has seen
  - `--all-branches` — current repo, every branch
  - `--harness <name>` — filter by which agent created the frame
- **Frames written before scope tagging existed have no `repo` field** — they get hidden by the default filter. If a query returns nothing in a populated capsule, retry with `--all-repos`.
- **Outside a git repo**, the CLI automatically falls back to `--all-repos`.
- **Disable globally** if you don't want any default scoping: `mvd config set scope.default off`.

### Automatic Memory Capture

After performing **significant work**, proactively store an observation. Significant work includes:

- Creating or editing files
- Running commands that produce meaningful output
- Discovering bugs or issues
- Making architectural decisions
- Finding solutions to problems
- Completing features or fixes

To store an observation, run:
```bash
echo '<CONTENT>' | ./scripts/mvd-put.sh "<SUMMARY>" "<TYPE>" "<TOOL>"
```

Where:
- `<CONTENT>` is a compressed description of what happened (key facts only, 2-5 sentences)
- `<SUMMARY>` is a one-line summary (e.g., "Fixed CORS bug in auth middleware")
- `<TYPE>` is one of: `discovery`, `decision`, `problem`, `solution`, `pattern`, `warning`, `success`, `refactor`, `bugfix`, `feature`
- `<TOOL>` is what triggered it (e.g., `file-edit`, `command`, `search`, `web-fetch`)

#### Capture Guidelines

- **DO capture**: Key decisions, bugs found/fixed, important file changes, architectural choices, error resolutions
- **DO NOT capture**: Trivial reads, listing directories, routine formatting, or anything the user explicitly asks you not to store
- **Compress aggressively**: Store key facts, not raw output. A 500-line file read → "Read auth.rs: implements JWT validation with RS256, expiry check, role-based claims"
- **Avoid duplicates**: Don't re-store the same observation within a single conversation

### Session Summary

Before the conversation ends (when wrapping up or the user says goodbye), generate a session summary. If Codex stop hooks are enabled they will also capture a lifecycle record, but you should still store a human-useful summary for substantive work:

1. Collect git changes if in a git repo:
   ```bash
   git diff --name-only HEAD 2>/dev/null | head -30
   git diff --cached --name-only 2>/dev/null | head -30
   git diff HEAD --stat 2>/dev/null | head -30
   ```
2. Store a summary:
   ```bash
   MVD_FILE=$(bash ./scripts/mvd-resolve.sh)
   echo '<SESSION_SUMMARY>' | mvd put "$MVD_FILE" --title "Session summary: <BRIEF_DESCRIPTION>" --label "session" --tag "summary"
   ```

The session summary should include what was accomplished, key decisions, files modified, and any unresolved issues.

### Memory Commands

When the user asks to query their memory, use these commands:

**Search memories (BM25 / lexical):**
```bash
./scripts/mvd-search.sh "<query>" 10
```

**Semantic vector search:**
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd vec "$MVD_FILE" --query "<query>" --limit 10 --json
```

**Ask questions (retrieval + synthesis):**
```bash
./scripts/mvd-ask.sh "<question>" 8
```

**View statistics:**
```bash
./scripts/mvd-stats.sh
```

Pass `--by-repo` to break frame counts down by repository / branch / harness:
```bash
MVD_FILE=$(bash ./scripts/mvd-resolve.sh) && mvd stats "$MVD_FILE" --by-repo
```

**View recent memories (timeline):**
```bash
./scripts/mvd-recent.sh 20
```

**Store a memory:**
```bash
echo '<content>' | ./scripts/mvd-put.sh "<summary>" "<type>" "<tool>"
```

**View a specific frame:**
```bash
./scripts/mvd-frame.sh <frame_id>
```

**Inspect mvd's own usage** (which harnesses, which commands, which repos):
```bash
./scripts/mvd-usage.sh                  # last 30 days, grouped by harness
./scripts/mvd-usage.sh --by command     # group by command name
./scripts/mvd-usage.sh --by repo        # group by repository
./scripts/mvd-usage.sh --since 7d --json
```

### Memory Skill — When to Use

Use the mvd memory system proactively when:
- The user asks about past work, decisions, or context
- You need to recall past solutions to similar problems
- The user explicitly asks to search, store, or review memories
- You want to check if a similar problem was encountered before

### Memory Types

When storing memories, classify them with the appropriate label:
- **discovery** — New information discovered
- **decision** — Important decision made
- **problem** — Problem or error encountered
- **solution** — Solution implemented
- **pattern** — Pattern recognized in code/data
- **warning** — Warning or concern noted
- **success** — Successful outcome
- **refactor** — Code refactoring done
- **bugfix** — Bug fixed
- **feature** — Feature added
- **session** — Session summary


## Global Context & CLI Features

**IMPORTANT NOTE FOR THE AGENT:**
`mvd` functions as a **global memory** spanning all projects being worked on. You can use it to query for ANY relevant information across projects, past sessions, or historical data. By default queries are filtered to the current repo + branch — pass `--all-repos` to broaden.

Recent CLI improvements you can utilize:
- `mvd vec <query>` — Cosine / semantic vector search.
- `mvd find <query>` — Exact-match BM25 / lexical search.
- `mvd ask <question>` — Retrieval + LLM synthesis (uses local Gemma 4).
- `mvd chat` — Interactive LLM REPL with replay-session history.
- `mvd memories` — Query declarative facts, extracted entities, slot properties.
- `mvd follow` — Traverse the Logic-Mesh entity relationship graph (e.g. `mvd follow --entity "System" --link "depends_on" --depth 2`).
- `mvd tables` — List and export extracted structured tables (CSV / JSON).
- `mvd schema` — Infer and list property schemas from memory records.
- `mvd session` — Time-travel session management.
- `mvd usage` — Telemetry: which harness has been calling mvd, which commands, which repos.
- `mvd stats --by-repo` — Frame counts broken down by repo / branch / harness.

### Local LLM (Gemma 4) — one-time `mvd setup`

`mvd ask`, `mvd chat`, and `mvd enrich --llm` use the local Gemma 4 model via mistral.rs. The first load quantizes the bf16 weights to Q4K in-place — a 30-120 s one-time cost on every fresh process. To avoid this, run `mvd setup` once: it pre-quantizes the model and writes the result to `~/.cache/memvid/llm/gemma-4-E4B-it/q4k-*.uqff`. Subsequent loads read the UQFF cache directly (~5 s).

If you see "Loading local LLM (...) from Hugging Face cache. Quantizing in-place..." it means the UQFF cache is not yet primed — suggest the user run `mvd setup`.
