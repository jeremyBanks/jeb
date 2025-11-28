# Copilot Instructions for jeb

## Before Completing Any Task

Always run these commands to ensure your changes are properly formatted, linted, and tested:

```bash
# Format code
cargo fmt

# Run clippy with automatic fixes where possible
cargo clippy --fix --allow-dirty --allow-staged

# Run all tests
cargo test

# Run all examples to regenerate their output
cd crates/jeb/examples && ./all.sh
```

## Code Style

- Follow Rust's standard formatting (enforced by `cargo fmt`)
- Address all clippy warnings
- Keep comments minimal - only add comments that explain _why_ something is done, not _what_ is done when the code is clear
- Doc comments (`///`) should be thorough since they're for people who can't see the code

## Error Handling

- Never use `from_utf8_lossy` - always use `from_utf8` with proper error handling via `?` or `.expect()`
- Prefer infallible APIs that return results with error/warning information rather than using `Result` types for expected failures
- Never call `process::exit()` directly - let errors propagate

## Testing

- Run tests with `cargo test`
- Examples in `crates/jeb/examples/` are self-regenerating scripts - run them to update their output
- Always run `crates/jeb/examples/all.sh` after making changes to ensure examples are up-to-date
