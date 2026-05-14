import os
import glob

# Harness directories
dirs = ["mvd-antigravity", "mvd-claude", "mvd-codex", "mvd-cursor", "mvd-cursor-local"]

# Find all markdown files in these directories
files_to_check = []
for d in dirs:
    for root, _, files in os.walk(d):
        for f in files:
            if f.endswith('.md') or f.endswith('.mdc') or f.endswith('.txt'):
                files_to_check.append(os.path.join(root, f))

# Section to append to core memory files
global_mem_note = """
## Global Context & CLI Features

**IMPORTANT NOTE FOR THE AGENT:**
`mvd` functions as a **global memory** spanning all projects being worked on. You can use it to query for ANY relevant information across projects, past sessions, or historical data. 

Recent CLI improvements you can utilize:
- `mvd vec <query>` — Perform Cosine/Semantic vector search.
- `mvd find <query>` — Perform exact-match BM25/Lexical search.
- `mvd chat` — Drop into an interactive LLM REPL that automatically maintains context history in a `ReplaySession`.
- `mvd memories` — Query declarative facts, extracted entities, and slot properties.
- `mvd follow` — Traverse the Logic-Mesh entity relationship graph (e.g., `mvd follow --entity "System" --link "depends_on" --depth 2`).
- `mvd tables` — List and export extracted structured tables (CSV/JSON).
- `mvd schema` — Infer and list property schemas from memory records.
- `mvd session` — Time-travel session management.
"""

for file_path in files_to_check:
    try:
        with open(file_path, 'r') as f:
            content = f.read()
            
        modified = False
        
        # Replace vec-search with vec
        if 'mvd vec-search' in content or 'vec-search' in content:
            content = content.replace('mvd vec-search', 'mvd vec')
            content = content.replace('vec-search', 'vec')
            modified = True
            
        # Add the global memory note to the core instruction files
        # Check if it's the main entry point or rule file
        basename = os.path.basename(file_path)
        if basename in ['mvd-memory.md', 'mvd-memory.mdc', 'AGENTS.md', 'CLAUDE.md']:
            if "Global Context & CLI Features" not in content:
                content += "\n" + global_mem_note
                modified = True
                
        if modified:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Updated {file_path}")
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}")

print("Update complete.")
