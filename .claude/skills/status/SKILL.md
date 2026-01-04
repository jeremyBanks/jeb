---
name: status
description: Fetch and display the brainstorm.md file from the claude/note-taking-tracker-wJTFN branch to show current project notes and status.
allowed-tools: Read, Bash
---

# Status Skill

## Overview

This skill fetches the `claude/note-taking-tracker-wJTFN` branch and displays the `brainstorm.md` file from that branch, providing quick access to project notes and status updates.

## When to Use This Skill

Invoke this skill when you want to:
- Check the current project status and notes
- View the brainstorming document
- See what's in the note-taking tracker branch

## Instructions

### Step 1: Fetch the Branch

Ensure the branch is available locally:

```bash
git fetch origin claude/note-taking-tracker-wJTFN:claude/note-taking-tracker-wJTFN 2>/dev/null || true
```

### Step 2: Display the File

Show the contents of `brainstorm.md` from that branch:

```bash
git show claude/note-taking-tracker-wJTFN:brainstorm.md
```

### Step 3: Handle Errors Gracefully

If the branch or file doesn't exist, provide a helpful error message explaining:
- The branch might not exist yet
- The file might not be present in that branch
- How to create the branch or file if needed

## Expected Output Format

Display the complete contents of the `brainstorm.md` file with a brief header explaining what's being shown:

```
# Project Status (from claude/note-taking-tracker-wJTFN)

[Contents of brainstorm.md]
```

If the file is not found, provide a clear error message and guidance.
