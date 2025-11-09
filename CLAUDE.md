# jeb (JSON Entity Bucket)

A flexible command-line tool for merging, formatting, and searching JSON data.

## Key Documentation

**Read these files before starting work:**
- **[README.md](README.md)** - Project overview, usage, and examples
- **[DESIGN.md](DESIGN.md)** - Architecture and design decisions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow and guidelines
- **[IDEAS.md](IDEAS.md)** - Future ideas and potential enhancements

## Critical Convention: Version Bumping

**⚠️ EVERY pull request MUST bump the version in `Cargo.toml`**

Current versioning scheme (while in 0.0.x):
- Patch version (0.0.X) - All changes while project is experimental
- Version stays at 0.0.x until project is stable (see README disclaimer)

CI will fail if version is not bumped!

## Common Commands

```bash
# Run all tests
cargo test

# Format code (run before committing)
cargo fmt

# Check formatting
cargo fmt --check

# Run linter (must pass with no warnings)
cargo clippy -- -D warnings

# Build release binary
cargo build --release

# Generate and view documentation
cargo doc --open
```

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add tests for new functionality
- Update documentation when adding public APIs

## Core Files

- `src/lib.rs` - Main library with JSON parsing, sorting, merging logic
- `src/main.rs` - CLI implementation
- `src/json_stream.rs` - Stream/text conversion utilities (work in progress)
