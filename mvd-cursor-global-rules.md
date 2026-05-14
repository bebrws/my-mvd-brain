---
description: Persistent memory via mvd — at session start, load context only for the current repo + branch. Captures stamp the current repo + branch. Broaden only when the user explicitly asks about other branches or repos.
alwaysApply: true
---

# MVD Memory System (Global Rule, Repo + Branch Scoped)

You have access to a **persistent, cross-project memory system** powered by `mvd`,
a single-file memory engine. Every observation, decision, discovery, and fix is
stored in a portable `.mv2` capsule.

This rule enforces **strict per-repo, per-branch scoping** so the agent feels
contextually aware of *where* the user is working — without dragging in
unrelated frames from other branches or projects.

This rule is **self-contained**: it does not require any helper scripts. All
shell snippets here use only the `mvd` CLI plus standard Unix tools.

---

## 0. Setup (verify silently, once per session)

`mvd resolve --ensure` is the single source of truth for the capsule path. It:

1. Picks the path — priority: `$MVD_FILE` env var → `$HOME/mvd.mv2` (if it
   exists, the shared global brain) → `./mvd/mvd.mv2` (per-repo fallback).
2. Creates the file (and parent dir) if missing.
3. Prints the resolved absolute path to stdout.

Run this **once per conversation, silently**:

```bash
MVD_FILE=$(mvd resolve --ensure 2>/dev/null) || {
  echo "WARN: mvd unavailable; skipping memory operations"
  return 0 2>/dev/null || true
}
export MVD_FILE
```

If `mvd` is not on `PATH` the command substitution fails, the `||` branch
fires, and the rest of the session continues without memory. **Never block
work on memory errors.**

Reuse `$MVD_FILE` in every subsequent command.

> Tip: if the user wants one global brain across every repo and `~/mvd.mv2`
> doesn't yet exist, run `MVD_FILE="$HOME/mvd.mv2" mvd resolve --ensure` once
> (or `mvd create ~/mvd.mv2`) to bootstrap it.

---

## 1. Session start — load context for THIS repo + branch only

At the **beginning of every conversation**, before answering, run these two
commands silently. Do **not** announce that you are running them.

```bash
# Default-scoped reads (no --all-repos, no --all-branches): mvd auto-detects
# the current git repo + branch from cwd and filters to those.
mvd timeline "$MVD_FILE" --limit 20 --reverse --json 2>/dev/null
mvd stats    "$MVD_FILE" --json                     2>/dev/null
```

**If the workspace is not a git repo**, do not load any context at session
start — there is no repo+branch to scope to and we do **not** want to dump
cross-repo memories the user didn't ask for. (`mvd` will auto-fall-back to
all-repos here; suppress that by skipping the load entirely.)

```bash
if git rev-parse --show-toplevel >/dev/null 2>&1; then
  mvd timeline "$MVD_FILE" --limit 20 --reverse --json 2>/dev/null
fi
```

The loaded context represents work done **on this repo, on this branch**. Use
it to inform your work and reference relevant past memories. The user should
feel that you remember what *they* did *here*.

---

## 2. Capture — frames are auto-stamped to THIS repo + branch

Before you finish a turn where you did **substantive** work, capture at least
one frame when **any** of these occurred:

- You created or changed files that affect behavior, APIs, or configuration
- You ran a command whose result matters (tests, build, lint, migration, deploy)
- You made an architectural / design decision
- You discovered a bug or shipped a fix the user would want recalled later
- You learned a non-obvious fact about the codebase, environment, or user

**Skip** capture for: trivial file reads, directory listings, pure formatting,
exploration that yielded nothing, or anything the user explicitly asked you not
to store.

`mvd put` automatically tags every new frame with the current git **repo**,
**branch**, and detected **harness** (cursor / claude-code / codex / aider /
shell). You do **not** need to set those flags — they're applied from the
working directory and environment unless you pass `--no-scope` or override with
`--repo`, `--branch`, `--harness`. Auto-stamping is what makes scoped reads
work — never disable it without a strong reason.

### Capture template

```bash
cat <<'CONTENT' | mvd put "$MVD_FILE" \
  --title "<ONE-LINE SUMMARY>" \
  --label "<TYPE>" \
  --tag   "<TOOL-OR-SOURCE>"
<2-5 sentences of compressed facts. Key file paths, function names,
the WHY (intent, tradeoffs, rejected options), and any follow-ups.>
CONTENT
```

### Required `--label` (memory type) — pick exactly one

| Label       | Use for                                              |
|-------------|------------------------------------------------------|
| `discovery` | New information about code, env, or behavior         |
| `decision`  | Architectural / design choice (always include WHY)   |
| `problem`   | Bug or error encountered                             |
| `solution`  | Fix applied to a prior `problem`                     |
| `pattern`   | Reusable pattern recognized in code or data          |
| `warning`   | Concern, gotcha, or "don't do X here"                |
| `success`   | Confirmed working outcome (tests pass, deploy OK)    |
| `refactor`  | Non-behavior-changing code reshape                   |
| `bugfix`    | Behavior-changing bug fix                            |
| `feature`   | New feature added                                    |
| `session`   | End-of-session summary (see §5)                      |
| `note`      | Manual user-requested "remember this"                |

### Suggested `--tag` values

`file-edit`, `command`, `search`, `web-fetch`, `manual`, `summary`, `mcp`,
`test`, `build`, `git`, `review`. Free-form is fine — keep it short.

### Compression rules

- **Compress aggressively.** Store the *signal*, not raw output.
- A 500-line file read → "Read `auth.rs`: implements JWT validation with RS256,
  expiry check, role-based claims."
- A 200-line test log → "Ran `cargo test`: 412 passed, 0 failed in 31s after
  fixing race in `wal::flush`."
- Always include WHY when capturing a `decision` — what was rejected and why.
- **No duplicates.** If you already captured this fact this turn, do not
  re-capture it.

---

## 3. Recall — when to query memory

Query memory **proactively** when:

- The user asks about past work, decisions, or context ("what did we decide…",
  "have I seen this before?", "remind me…")
- You are about to solve a problem that may have been solved before
- You are about to make a decision the user has likely already made
- The user references "the X project", "last week", "that bug", or any
  pronoun-only reference to prior work

Default reads are scoped to the **current repo + branch**. Read §4 for when
and how to broaden.

### Search (lexical / BM25 — exact terms, code identifiers)

```bash
mvd find "$MVD_FILE" --query "<keywords>" --top-k 10 --json
```

### Vector search (semantic — concepts, paraphrases)

```bash
mvd vec "$MVD_FILE" --query "<natural language>" --limit 10 --json
```

### Synthesizing answers (no LLM)

After running `mvd find` / `mvd vec`, **you** synthesize the answer yourself
from the returned frames. Cite frames by title and timestamp when relevant. If
nothing useful comes back at the default scope, consider broadening per §4
before saying so — do not fabricate. Do **not** call `mvd ask` (it would
invoke the local LLM) — see §6 rule #10.

### View a specific frame

```bash
mvd view "$MVD_FILE" --frame-id <id>
```

### Recent timeline (current repo + branch)

```bash
mvd timeline "$MVD_FILE" --limit 20 --reverse --json
```

---

## 4. Scope decision matrix — when to broaden

You always **start** at the narrowest scope (current repo + branch). Only
broaden when the user's question or context demands it.

| User intent | What to do | Flags |
|---|---|---|
| Default — anything tied to "here" / "this branch" / "what we just did" | Stay narrow | *(none)* |
| "What's been done on this repo?" / "have I touched X anywhere in this project?" | Broaden to repo-wide | `--all-branches` |
| "What did I do on the `<other>` branch?" / "remind me what happened in `feature/auth`" | Switch branches | `--branch <name>` |
| "What auth libraries have I used?" / "have I solved this before, anywhere?" | Cross-repo | `--all-repos` (alias `--global`) |
| User explicitly names another project | Cross-repo with target | `--repo <id>` |

When you broaden, **say so in your response** — "Searching this repo across all
branches…" or "Looking at every project you've worked on…" — so the user knows
the scope changed.

### Examples

```bash
# This repo, every branch:
mvd find "$MVD_FILE" --query "rate limit" --all-branches --top-k 10 --json

# A specific other branch on this repo:
mvd find "$MVD_FILE" --query "rate limit" --branch develop --top-k 10 --json

# Cross-repo, current branch is irrelevant:
mvd find "$MVD_FILE" --query "ratelimit middleware" --all-repos --top-k 10 --json

# A specific other repo:
mvd find "$MVD_FILE" --query "auth implementation" --repo github.com/me/other --top-k 10 --json

# Filter by which agent recorded the frame:
mvd find "$MVD_FILE" --query "deploy" --harness claude-code --top-k 10 --json
```

### Critical: legacy unscoped frames

Frames written **before** scope tagging existed have no `repo` field and are
**excluded by default**. That's intentional — they pollute repo-scoped recall.
If a query feels suspiciously empty in a populated capsule and the user asks
about historical context, retry with `--all-repos` to surface them.

### Inspect what's been writing to the capsule

`mvd usage` reads the local invocation log (`~/.mvd/usage.jsonl`) plus the
capsule's scope tags:

```bash
mvd usage --since 30d                   # last 30 days, by harness (default)
mvd usage --by command                  # which commands have run
mvd usage --by repo                     # which repos have written / queried
mvd usage --harness cursor --since 7d   # filter to one harness
mvd usage --frames                      # derived from capsule (writes only)
```

Frame counts broken down by scope are also available via `mvd stats`:

```bash
mvd stats "$MVD_FILE" --by-repo
```

### Auxiliary CLI features (use when relevant)

| Command         | Use                                                      |
|-----------------|----------------------------------------------------------|
| `mvd memories`  | Declarative facts, extracted entities, slot properties   |
| `mvd follow`    | Traverse entity relationships (depends_on, etc.)         |
| `mvd tables`    | List / export structured tables (CSV/JSON)               |
| `mvd schema`    | Inferred property schemas                                |
| `mvd session`   | Time-travel session management                           |
| `mvd enrich`    | Rules-engine fact extraction — **never pass `--llm`**    |
| `mvd usage`     | Telemetry: harness / command / repo usage                |

---

## 5. Session summary — before the conversation ends

When the user signals end-of-session ("thanks", "goodbye", "we're done",
"wrap up"), capture a session-summary frame. It will be auto-stamped to the
current repo + branch.

```bash
CHANGED=$(git diff --name-only HEAD 2>/dev/null | head -20)
cat <<CONTENT | mvd put "$MVD_FILE" \
  --title "Session summary: <BRIEF DESCRIPTION>" \
  --label "session" \
  --tag   "summary"
What was accomplished:
  - <bullets>

Key decisions:
  - <bullets, each with WHY>

Files modified:
$CHANGED

Unresolved / follow-ups:
  - <bullets>
CONTENT
```

If not in a git repo, omit the `Files modified` block.

---

## 6. Hard rules (do not violate)

1. **Never block the user on memory errors.** If `mvd` fails, log internally and
   continue the actual task.
2. **Never store secrets.** Skip capture if the content includes API keys,
   tokens, passwords, private keys, or anything matching `*_SECRET`, `*_KEY`,
   `Authorization:`, `Bearer …`, or `-----BEGIN … PRIVATE KEY-----`.
3. **Never dump raw command output** into a frame. Summarize.
4. **Never re-capture** the same fact twice in one turn.
5. **Always set `--label` and `--title`.** A frame without classification is
   close to useless.
6. **Run session-start load (§1) silently** — do not narrate it to the user.
7. **Honor opt-outs.** If the user says "don't remember this", "stop saving",
   or similar, skip all captures for the remainder of the conversation.
8. **Default to narrow scope (§4).** Read at current repo + branch by default;
   broaden only when the user's question explicitly spans branches or repos,
   and announce when you do.
9. **Skip session-start context loading entirely when not in a git repo.** No
   repo means no scope to filter on; do not fall back to `--all-repos` for the
   silent session-start load.
10. **Never invoke the local LLM.** Do not run `mvd ask`, `mvd chat`, or
    `mvd enrich --llm`. Synthesize answers from `mvd find` / `mvd vec`
    results yourself. If you run `mvd enrich`, omit the `--llm` flag — the
    default rules engine is what you want.

---

## 7. Quick reference (copy-paste cheat sheet)

```bash
# Resolve capsule (creates it if missing; fails fast if mvd not on PATH)
MVD_FILE=$(mvd resolve --ensure 2>/dev/null) || { echo "WARN: mvd unavailable"; return 0 2>/dev/null || true; }
export MVD_FILE

# Session start — current repo + branch only; skip if not in a git repo
if git rev-parse --show-toplevel >/dev/null 2>&1; then
  mvd timeline "$MVD_FILE" --limit 20 --reverse --json
fi
mvd stats "$MVD_FILE" --json

# Capture (auto-stamps repo / branch / harness — never override unless asked)
echo '<facts>' | mvd put "$MVD_FILE" --title '<summary>' --label '<type>' --tag '<source>'

# Recall — narrow by default; broaden by user intent (see §4)
mvd find "$MVD_FILE" --query '<kw>'      --top-k 10 --json                  # this repo + branch
mvd find "$MVD_FILE" --query '<kw>'      --all-branches  --top-k 10 --json  # this repo, every branch
mvd find "$MVD_FILE" --query '<kw>'      --branch other  --top-k 10 --json  # specific other branch
mvd find "$MVD_FILE" --query '<kw>'      --all-repos     --top-k 10 --json  # everything
mvd find "$MVD_FILE" --query '<kw>'      --repo <id>     --top-k 10 --json  # specific other repo
mvd vec  "$MVD_FILE" --query '<concept>' --limit 10 --json
mvd timeline "$MVD_FILE" --limit 20 --reverse --json
mvd view "$MVD_FILE" --frame-id <id>

# Inspect usage (which harness / command / repo)
mvd usage --since 30d
mvd stats "$MVD_FILE" --by-repo
```

---

**Bottom line:** load context for the **current repo + current branch** only at
session start (skip entirely if not in a git repo). Capture substantive work as
you go — auto-stamping keeps every frame tied to where it happened. Default
recall is narrow. Broaden — to repo-wide, to a specific other branch, or
across every repo — **only** when the user's question makes that the right
scope, and tell them when you do. Summarize at the end.
