---
name: dig-through-history-pit
description: Recover deleted files from git history and browse recovered content
allowed-tools: Read, Grep, Glob, Bash
---

# Dig History Pit

Two capabilities:

1. **Recover deleted files** from git history into the `history-pit/` directory
2. **Search git history** for commits that added or removed specific text

## Usage

```
/dig-history-pit [file patterns or search text]
```

Examples:

- `/dig-history-pit *.md` - Recover deleted markdown files
- `/dig-history-pit *.rs *.toml` - Recover multiple file types
- `/dig-history-pit Z85 encoding` - Search for text "Z85 encoding" in git history
- `/dig-history-pit "function foo"` - Search for specific text in git history

## Workflow

The recommended approach is to **start with text search**, then use the results
to inform targeted file recovery if needed.

## Step 1: Text Search (git log -S)

Unless the user provides an explicit glob pattern (like `*.md`), start by
searching git history for the text:

```bash
git log --all -S "SEARCH_TEXT" --oneline --reverse | head -20
```

This shows commits (oldest first) where the text was added or removed.

### Analyze the Results

For interesting commits, show details:

```bash
git show COMMIT_HASH --stat
git log --format="%H %ci %s" COMMIT_HASH -1
```

To see the text in context:

```bash
git show COMMIT_HASH -p | head -200
```

Key things to extract:
- **Which files** contained the text
- **Whether those files still exist** in HEAD
- **File extensions** of the relevant files

## Step 2: Targeted File Recovery (if needed)

If the text search reveals deleted files the user wants to recover, use the
dig-history-pit tool with a **targeted glob pattern** based on what you learned:

- If the text was in `docs/old-feature.md` (deleted), recover with `*.md`
- If it was in `src/utils/encoder.rs`, recover with `*.rs`
- If multiple file types, combine: `*.md *.rs`

This is more efficient than recovering all files of a type - the text search
narrows down exactly what to look for.

## File Recovery Tool Details

Execute the dig-history-pit tool:

```bash
cargo run --bin dig-history-pit -- [patterns]
```

If no patterns specified, the default is `*.md`.

The tool will:

- Scan git history for deleted files matching the pattern(s)
- Filter out blobs that still exist in HEAD (content isn't truly lost)
- Skip files from `history-pit/` paths (avoid re-recovering)
- Filter out empty/whitespace-only files (unless they're the only version)
- Recover both the oldest and newest versions of each deleted file

### Output Location

Files are recovered to `history-pit/PATTERN/` where PATTERN is derived from the
glob (e.g., `*.md` -> `_md/`, `*.rs *.toml` -> `_rs__toml/`).

### Filename Format

`YYYYMMDD[abbrev]-COMMIT-BLOBHASH-flattened-path.ext`

- `YYYYMMDD` - Creation date (when this content first appeared at this path)
- `[abbrev]` - Abbreviated deletion date (omitted if same day, DD if same month,
  MMDD if same year, full YYYYMMDD if different year)
- `COMMIT` - 6-char prefix of the commit that deleted the file
- `BLOBHASH` - 8-char prefix of the blob hash (identifies content)
- `flattened-path` - Original path with `/` replaced by `-`

### Exploring Recovered Files

After recovery, help the user:

- List recovered files matching certain patterns
- Read specific recovered files
- Compare different versions of the same file (same path, different blob hashes)
- Search for content within recovered files

## Important Caveats

**This is NOT a complete history of all file versions.** The tool only recovers:

1. **Deleted files** - If a file exists in HEAD, its historical content isn't
   recovered (it's available via `git log`/`git show`)
2. **Oldest and newest deletions** - If the same content at the same path was
   deleted multiple times, only the first and last deletion are recovered
3. **Non-whitespace content** - Empty or whitespace-only files are skipped
   unless they're the only version for that path
4. **Files outside history-pit/** - Previously recovered files are not
   re-recovered

For complete file history, use standard git commands:

```bash
git log --follow -p -- path/to/file
```

## Technical Details

The tool uses `git log --raw` with `-m` flag to properly handle merge commits.
It compares blob hashes against HEAD using full 40-character hashes to ensure
accurate filtering.

Recovered files preserve their original content exactly as it existed when
deleted.
