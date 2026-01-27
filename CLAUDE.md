# Claude Code Notes

## CRITICAL: Save frequently!

Run `git add -A && git commit -m "..."` (or just `save`):
- BEFORE starting any changes
- AFTER completing any changes
- OFTEN in between for complex changes
- We need many snapshots to safely experiment

## Project: zipng

Polyglot PNG+ZIP file generator. Key constraints:
- Max 60KB per file (IDAT boundary limitation)
- Files are sorted lexicographically
- Labels use size-appropriate fonts (Sky < 128K, Sugimori < 512K, Mini < 1M, Micro < 3M)
