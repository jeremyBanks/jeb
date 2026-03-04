# git-zoom

Extract and embed subtrees within git repositories while preserving history.

## Overview

`git-zoom` lets you temporarily "zoom into" a subdirectory of your repo, making it appear as if it's the entire repository. When you're done, you can "zoom out" and embed your changes back into the original structure.

This is useful for:
- Working on a subdirectory without the noise of the full monorepo
- Sending a subset of your repo to an agent that might get confused by extra files
- Creating a clean view of a single crate/package for focused work

## Installation

```bash
cargo install --path crates/git-zoom
```

## Usage

### Zoom In

Extract a subtree to become the working directory:

```bash
# Zoom into crates/my-crate/
git-zoom in crates/my-crate

# Allow zooming into non-existent path (creates empty tree)
git-zoom in new/path --allow-empty
```

After zooming in:
- Your working directory contains only files from the subtree
- Other files are preserved in git history (in a special merge commit)
- You can work, commit, and push as normal

### Zoom Out

Embed the current tree back into the full repository:

```bash
# Zoom out, merging changes back
git-zoom out

# Zoom out to a specific target (useful if zoomed branch was pushed)
git-zoom out main

# Error if subtree would be empty
git-zoom out --deny-empty
```

## How It Works

1. **Zoom in** creates a merge commit that preserves the full tree in one parent while making the subtree the working tree
2. The `.gitignore` is updated to ignore files outside the subtree path
3. **Zoom out** creates another merge that embeds the subtree back into the full tree structure
4. All history is preserved — no data is lost

## Requirements

- Must be run from the repository root
- Working tree must be clean (no uncommitted changes)
- Git repository must exist

## Example Workflow

```bash
# Start in a monorepo
ls
# Cargo.toml  crates/  docs/  scripts/

# Zoom into a specific crate
git-zoom in crates/my-crate

ls  
# Cargo.toml  src/  tests/

# Work on the crate...
vim src/lib.rs
git commit -am "Add feature"

# Zoom back out
git-zoom out

ls
# Cargo.toml  crates/  docs/  scripts/
# Changes are now in crates/my-crate/
```

## Caveats

- Don't use `git-zoom` if you're in the middle of a rebase or merge
- Make sure any agents you share a zoomed repo with have `git-zoom` installed to zoom back out
- The zoom commits have special markers; don't manually manipulate them
