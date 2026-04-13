# MVD Cursor

**Persistent memory for AI coding agents — powered by a single portable file.**

Give your Cursor agent long-lived memory across sessions. Observations, decisions, and fixes can be stored in one `.mv2` capsule and searched with `mvd`.

## Prerequisites

- [Cursor](https://cursor.com)
- [`mvd`](https://github.com/memvid/memvid) on your `PATH` (this repo’s Apache-licensed CLI or another compatible build)

## Choose an install style

| Goal | Where things live | Best for |
|------|-------------------|----------|
| **Per-project** | `your-repo/.cursor/`, `your-repo/scripts/` | Team shares the same rules via git; memory can be per-repo under `./mvd/mvd.mv2` |
| **Global (`~/.cursor`)** | `~/.cursor/rules/`, `~/.cursor/hooks/`, `~/.cursor/scripts/mvd/` | Same behavior in **every** workspace; pair with `~/mvd.mv2` for one brain everywhere |

You can use **both**: global hooks capture events everywhere; a project can still add extra rules under its own `.cursor/rules/`.

### Memory file resolution

Helper scripts resolve the capsule in this order:

1. **`$HOME/mvd.mv2`** if it exists (shared across projects)
2. Otherwise **`./mvd/mvd.mv2`** next to the project (create with `mvd create` / `mvd-ensure.sh`)

```bash
mvd create ~/mvd.mv2   # optional: one global capsule for all repos
```

### Rules vs hooks (important)

- **Rules** (`.mdc` files) only **tell the model** what to do. They do **not** run shell commands by themselves.
- **Hooks** (`hooks.json` + `mvd-hook-handler.sh`) run **outside** the model and call `mvd put` after edits, shell commands, MCP calls, session end, etc.

For hands-off persistence, install hooks (per-project or global). Rules still help the agent load the timeline at session start and run extra `mvd put` when hooks do not capture reasoning.

---

## Install A — Per-project

Copies rules, hooks, and scripts next to your repository root.

### Option 1: install script (from this directory)

```bash
cd /path/to/memvid/mvd-cursor   # or wherever this folder lives
./install-project.sh /path/to/your/project
```

### Option 2: manual copy

```bash
cp -R mvd-cursor/.cursor /path/to/your/project/
cp -R mvd-cursor/scripts     /path/to/your/project/
chmod +x /path/to/your/project/.cursor/hooks/*.sh
chmod +x /path/to/your/project/scripts/*.sh
```

### Layout after install

```
your-project/
├── .cursor/
│   ├── hooks.json              # registers Cursor hooks (paths from project root)
│   ├── hooks/
│   │   ├── mvd-hook-handler.sh
│   │   └── mvd-resolve-global.sh
│   └── rules/
│       ├── mvd-memory.mdc      # alwaysApply — load + agent capture policy
│       └── mvd-*.mdc           # agent-requested helpers (search, ask, …)
├── scripts/
│   ├── mvd-resolve.sh
│   ├── mvd-ensure.sh
│   ├── mvd-capture.sh
│   └── mvd-put.sh
└── mvd/
    └── mvd.mv2                 # created on demand if no ~/mvd.mv2
```

Open the **project folder** in Cursor. Restart Cursor after the first hook install so `hooks.json` is picked up.

Hook commands in `.cursor/hooks.json` are relative to the **project root**, for example:

`.cursor/hooks/mvd-hook-handler.sh`

---

## Install B — Global (`~/.cursor`, all workspaces)

Installs rules, hooks, and scripts under your user Cursor config so **every** session sees them.

### Option 1: install script (recommended)

From the `mvd-cursor` directory:

```bash
cd /path/to/memvid/mvd-cursor
./install-global.sh
```

This script:

- Copies `scripts/*.sh` → `~/.cursor/scripts/mvd/`
- Copies hook scripts → `~/.cursor/hooks/`
- Copies `global-user/hooks.json` → `~/.cursor/hooks.json` (uses `./hooks/mvd-hook-handler.sh` relative to `~/.cursor/`)
- Rewrites every `mvd-*.mdc` rule so `bash ./scripts/…` becomes `bash "$HOME/.cursor/scripts/mvd/…"` and writes them to `~/.cursor/rules/`

Then **restart Cursor**. Confirm **Settings → Hooks** and the **Hooks** output channel if anything fails.

### Option 2: manual global install

```bash
mkdir -p ~/.cursor/scripts/mvd ~/.cursor/hooks ~/.cursor/rules

cp mvd-cursor/scripts/*.sh ~/.cursor/scripts/mvd/
chmod +x ~/.cursor/scripts/mvd/*.sh

cp mvd-cursor/.cursor/hooks/mvd-hook-handler.sh \
   mvd-cursor/.cursor/hooks/mvd-resolve-global.sh \
   ~/.cursor/hooks/
chmod +x ~/.cursor/hooks/mvd-hook-handler.sh ~/.cursor/hooks/mvd-resolve-global.sh

cp mvd-cursor/global-user/hooks.json ~/.cursor/hooks.json
```

Rules in the repo assume `./scripts/` (project-relative). For global use, either:

- Run `./install-global.sh` (rewrites paths), **or**
- Copy `mvd-cursor/.cursor/rules/*.mdc` to `~/.cursor/rules/` and replace every `bash ./scripts/` with `bash "$HOME/.cursor/scripts/mvd/"` in each file.

---

## After installation

1. Ensure `mvd` works: `mvd version` (or your build’s equivalent).
2. Create a capsule if you want global memory: `mvd create ~/mvd.mv2`.
3. Restart Cursor after changing `hooks.json` or hook scripts.
4. Optional: trim noisy hooks — edit `hooks.json` and remove the `afterAgentResponse` entry if you do not want a frame per long assistant message.

---

## Rules reference

| Rule | Role |
|------|------|
| `mvd-memory.mdc` | `alwaysApply`: session start timeline/stats, mandatory agent captures when hooks are not enough, session summary |
| `mvd-stats.mdc` | User asks for memory / file stats |
| `mvd-search.mdc` | Search memories (`mvd find`) |
| `mvd-ask.mdc` | Questions about past work (`mvd ask` / find) |
| `mvd-recent.mdc` | Recent timeline |
| `mvd-remember.mdc` | Explicit “remember this” |
| `mvd-session-summary.mdc` | End-of-session summary |

### Natural-language triggers

- “What do you remember about …?” → ask / search rules  
- “Search my memory for …” → search  
- “Recent activity / timeline” → recent  
- “Remember that …” → remember  
- “Session summary” → session-summary rule  

---

## Helper scripts

| Script | Purpose |
|--------|---------|
| `mvd-resolve.sh` | Resolve capsule path (`~/mvd.mv2` vs `./mvd/mvd.mv2`) |
| `mvd-ensure.sh` | Create the local capsule if missing |
| `mvd-capture.sh` | Classify and `mvd put` from stdin |
| `mvd-put.sh` | Thin wrapper around `mvd put` |

Hook-side resolution for hooks that may run with an arbitrary cwd uses `mvd-resolve-global.sh` (uses `$HOME` and `$CURSOR_PROJECT_DIR`).

---

## Package layout (`mvd-cursor/`)

```
mvd-cursor/
├── README.md                 # this file
├── install-project.sh        # copy .cursor + scripts into a repo
├── install-global.sh         # install into ~/.cursor (rules + hooks + scripts)
├── global-user/
│   └── hooks.json            # template for ~/.cursor/hooks.json (./hooks/… paths)
├── .cursor/
│   ├── hooks.json            # template for <project>/.cursor/hooks.json
│   ├── hooks/
│   │   ├── mvd-hook-handler.sh
│   │   └── mvd-resolve-global.sh
│   └── rules/
│       └── mvd-*.mdc
└── scripts/
    ├── mvd-resolve.sh
    ├── mvd-ensure.sh
    ├── mvd-capture.sh
    └── mvd-put.sh
```

---

## FAQ

<details>
<summary><b>How big is the memory file?</b></summary>

Empty: on the order of tens of KB. Grows roughly with stored text. Typical personal use stays small.

</details>

<details>
<summary><b>Is it private?</b></summary>

Fully local: the `.mv2` file is on disk. What you send to cloud models is still governed by Cursor and your settings.

</details>

<details>
<summary><b>Hooks run but I see no new frames</b></summary>

Check that `mvd` is on `PATH` for GUI-launched Cursor, that `mvd put` works in a terminal, and the Hooks output channel for errors. The handler intentionally exits 0 even when `mvd` fails so the agent is never blocked.

</details>

<details>
<summary><b>Reset project-local memory</b></summary>

```bash
rm -rf ./mvd/
```

</details>

<details>
<summary><b>How is this different from Cursor’s built-in memory UI?</b></summary>

MVD stores a **portable file** you can commit, copy, or encrypt (`mvd lock` / `mvd unlock` where supported). It is separate from Cursor’s app-managed notepads.

</details>

---

Built on **[memvid](https://github.com/memvid/memvid)** — the single-file memory engine for AI agents.
