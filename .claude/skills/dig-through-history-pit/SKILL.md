---
name: dig-through-history-pit
description: Recover deleted files from git history and browse recovered content
allowed-tools: Read, Grep, Glob, Bash
---

# Dig History Pit

Recover deleted files from git history into the `history-pit/` directory, then
help the user browse and explore the recovered content.

## Usage

```
/dig-history-pit [optional: file patterns]
```

Examples:

- `/dig-history-pit` - Recover deleted markdown files (default: `*.md`)
- `/dig-history-pit *.rs` - Recover deleted Rust files
- `/dig-history-pit *.md *.toml` - Recover multiple file types

## Instructions

When this skill is invoked:

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
