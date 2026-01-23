# Future Topics for Discussion

## Rollback Handling

How should we handle reverting commits made with `save`?

Considerations:

- Commit messages encode metadata (revision index, origin, etc.)
- Reverting disrupts the linear sequence
- Do we need special logic to detect reverts?
- Should reverted commits affect the next commit's metadata?
- How do we handle revert chains?

## History Normalization

How should we normalize/rebuild history when needed?

The `--rebuild` flag exists to ignore all commit messages and recalculate from
scratch.

Questions:

- When should users run `--rebuild`?
- Should we detect when history looks "wrong" and suggest rebuild?
- Can we incrementally fix history without full rebuild?
- How do we handle repositories with mixed `save` and manual commits?
- Should rebuild preserve the tree/changes while fixing metadata?
