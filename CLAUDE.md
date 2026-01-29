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

Polyglot PNG+ZIP file generator. Unless otherwise instructed:
- Focus on `crates/zipng` and related files
- All examples should write their output to a subdirectory of `target/` in the repo root

Key constraints:
- Max 60KB per file (IDAT boundary limitation)
- Files are sorted lexicographically
- Labels use size-appropriate fonts (Sky < 128K, Sugimori < 512K, Mini < 1M, Micro < 3M)

## Code Documentation Requirements

When implementing code with subtle algorithms (especially in polyglot/mod.rs):
- Add detailed comments explaining the logic, constraints, and invariants
- Include ASCII diagrams where they help clarify data structures or byte layouts
- Comments can be extensive if needed to fully explain the algorithm
- ALWAYS consult existing comments before making changes
- ALWAYS update comments after making changes to keep them accurate
- Document any non-obvious relationships between filter bytes, DEFLATE blocks, and ZIP structures

## Bash Command Guidelines

- Run simple commands directly (git status, cargo test, etc.)
- NEVER write to /tmp/ - use project directories instead
- NEVER chain multiple commands with pipes or && in complex ways
- For multi-step scripts or test files: write to a file (e.g., examples/debug_foo.rs) so approval is needed once
- Re-use the same debug file for one-off tests rather than creating many files
