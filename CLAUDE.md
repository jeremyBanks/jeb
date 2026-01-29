# Claude Code Instructions

## Git Workflow

### Committing Changes

- Use `save` instead of `git commit` for quick snapshots during work
- Arguments are optional - no commit message needed for mid-work snapshots
- Use `save --empty --message "description"` to add a message to a previous snapshot later

### CRITICAL: Never Edit Git History

**NEVER EVER EVER EVER EVER edit existing git history under any circumstance.**

This means:
- No `git rebase -i`
- No `git commit --amend`
- No `git reset --hard` to previous commits
- No `git push --force`
- No squashing, reordering, or modifying existing commits

If you need to fix something, create a NEW commit. History is sacred.

## Image Comparison

When comparing image files (e.g. verifying PNG output), use the image_diff example:

    cargo run --example image_diff -- <old> <new>

This provides a text summary of differences suitable for CI/agent use.
