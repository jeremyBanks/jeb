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

## Detecting Search Mode

Determine which mode based on the argument:

- **Glob pattern** (contains `*` or looks like a file extension like `.rs`) →
  run dig-history-pit tool for file recovery
- **Plain text** (words, phrases, or quoted strings) → run `git log -S` text
  search

## Instructions

When this skill is invoked, first determine which mode to use based on the
argument (see "Detecting Search Mode" above), then follow the appropriate
section below.

---

## Mode A: Text Search (git log -S)

Use this mode when the argument is plain text (not a glob pattern).

### 1. Search for Commits

Run the search to find commits that added or removed the text:

```bash
git log --all -S "SEARCH_TEXT" --oneline --reverse | head -20
```

This shows commits in chronological order (oldest first) where the text was
added or removed.

### 2. Show Details of Relevant Commits

For the commits of interest (typically the oldest one where text first
appeared), show details:

```bash
git show COMMIT_HASH --stat
git log --format="%H %ci %s" COMMIT_HASH -1
```

You can also show the actual diff to see the text in context:

```bash
git show COMMIT_HASH -p | head -200
```

### 3. Help the User Explore

Based on the search results:

- Show which files contained the text
- Offer to check out or display specific versions
- Help trace how the text evolved through history
- If the file was deleted, suggest using file recovery mode

---

## Mode B: File Recovery (dig-history-pit tool)

Use this mode when the argument contains glob patterns (like `*.md` or `*.rs`).

### 1. Run the Recovery Tool

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

### 2. Explain the Output

Files are recovered to `history-pit/PATTERN/` where PATTERN is derived from the
glob (e.g., `*.md` -> `_md/`, `*.rs *.toml` -> `_rs__toml/`).

**Filename format**: `YYYYMMDD[abbrev]-COMMIT-BLOBHASH-flattened-path.ext`

Components:

- `YYYYMMDD` - Creation date (when this content first appeared at this path)
- `[abbrev]` - Abbreviated deletion date (omitted if same day, DD if same month,
  MMDD if same year, full YYYYMMDD if different year)
- `COMMIT` - 6-char prefix of the commit that deleted the file
- `BLOBHASH` - 8-char prefix of the blob hash (identifies content)
- `flattened-path` - Original path with `/` replaced by `-`

### 3. Help the User Explore

After recovery, help the user find what they're looking for:

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
