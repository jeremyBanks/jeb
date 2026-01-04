---
description: Summarize recent changes and brainstorming.
argument-hint: [focus]
---

You will be provided with git status, some recent commits, and some recent
changes to a brainstorming notes branch. You may be provided with a specific
user query at the bottom. If there's no specific query, just summarize the
provided information for the user, but if the user asks a question, or provides
a subject of focus, for which you require additional information, run as many
commands as necessary, as long as you DO NOT update/edit/delete any files or
edit any git state or other state in any systems. (You may fetch from git
remotes, but DO NOT update any local branches or change HEAD, the index, or the
working tree.)

# Brainstorming

```
git fetch origin claude/note-taking-tracker-wJTFN
```

!`git fetch origin claude/note-taking-tracker-wJTFN`

```
git show origin/claude/note-taking-tracker-wJTFN:brainstorm.md |
  grep \
    --before=2 \
    --after=2 \
    --line-number \
    --regexp="📌" \
    --regexp="⭐" \
    --regexp="🔖"
```

!`git show origin/claude/note-taking-tracker-wJTFN:brainstorm.md | grep --before=2 --after=2 --line-number --regexp="📌" --regexp="⭐" --regexp="🔖"`

```
git diff \
    --unified=2 \
    "$( \
        git merge-base \
            origin/claude/note-taking-tracker-wJTFN@'{2 days ago}' \
            origin/claude/note-taking-tracker-wJTFN~8 \
    )" \
    origin/claude/note-taking-tracker-wJTFN \
    -- \
    brainstorm.md
```

!`git diff --unified=2 "$(git merge-base origin/claude/note-taking-tracker-wJTFN@'{2 days ago}' origin/claude/note-taking-tracker-wJTFN~8)" origin/claude/note-taking-tracker-wJTFN -- brainstorm.md`

# Local Changes

```
git status
```

!`git status`

```
git log --graph --oneline --decorate -64
```

!`git log --graph --oneline --decorate -64`

```
git log --first-parent --decorate -16
```

!`git log --first-parent --decorate -16`

# User Query (if any)

$ARGUMENTS

---
