# Claude Code Notes

## CRITICAL: Save frequently!

Use `save` command (NOT `git commit` directly):
- `save` - snapshot without message (for quick mid-work saves)
- `save --message "description"` - snapshot with message
- `save --empty --message "description"` - add message to previous empty commit

NEVER EDIT EXISTING GIT HISTORY (no amend, no rebase, no force push)

Save:
- BEFORE starting any changes
- AFTER completing any changes
- OFTEN in between for complex changes
- We need many snapshots to safely experiment

## Project: zipng

Polyglot PNG+ZIP file generator. Key constraints:
- Max 60KB per file (IDAT boundary limitation)
- Files are sorted lexicographically
- Labels use size-appropriate fonts (Sky < 128K, Sugimori < 512K, Mini < 1M, Micro < 3M)
