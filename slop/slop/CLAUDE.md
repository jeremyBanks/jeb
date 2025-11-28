# jeb (JSON Entity Bucket)

A flexible command-line tool for merging, formatting, and searching JSON data.

## Workspace Structure

This project uses a Cargo workspace with multiple crates:

- **`crates/jeb`** - Main CLI tool and library
- **`crates/json-encoded-binary`** - JEB85 binary encoding library

## Key Documentation

**Read these files before starting work:**

- **[README.md](README.md)** - Project overview, usage, and examples
- **[DESIGN.md](DESIGN.md)** - Architecture and design decisions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow and guidelines
- **[IDEAS.md](IDEAS.md)** - Future ideas and potential enhancements

## Critical Convention: Version Bumping

**⚠️ EVERY pull request MUST bump the version in root `Cargo.toml` (workspace
version)**

Current versioning scheme (while in 0.0.0-vibes-YYYY-MM-DD.xxxx):

- Version format: 0.0.0-vibes-YYYY-MM-DD.xxxx (date-based versioning)
- Use `./scripts/new-version.sh` to generate a new version based on current
  date/time
- Version stays at 0.0.0-vibes-* until project is stable (see README disclaimer)
- Crates inherit the workspace version

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

## Running CI Checks Locally

**TL;DR**: Use `scripts/ci-local.sh` to run all CI checks locally before
pushing, catching failures early and saving time.

### Why run CI locally?

- **Catch failures early**: Find CI issues on your machine instead of in GitHub
  Actions
- **Save time**: No need to commit/push/wait to see if tests pass
- **Faster iteration**: Test changes immediately
- **Free CI minutes**: Local runs don't consume GitHub Actions minutes

### Quick Start

Run all CI checks that GitHub Actions will run:

```bash
./scripts/ci-local.sh
```

This script runs (in order):

1. **Format check** (`cargo fmt --check`) - Ensures code is formatted correctly
2. **Linter** (`cargo clippy`) - Catches common mistakes and enforces best
   practices
3. **Tests (debug)** (`cargo test`) - Runs all tests in debug mode
4. **Tests (release)** (`cargo test --release`) - Runs all tests in optimized
   release mode
5. **Release build** (`cargo build --release`) - Ensures project builds in
   release mode

The script exits immediately on first failure, showing you exactly what needs to
be fixed.

### Skipping the Release Build

The release build can be slow. Skip it during rapid iteration:

```bash
SKIP_BUILD=1 ./scripts/ci-local.sh
```

### Checking Version Format

Verify your version follows project conventions:

```bash
./scripts/check-version.sh
```

This checks that:

- Version is in `0.0.0-vibes-YYYY-MM-DD.xxxx` format (required for this project)
- Version has been bumped from the base branch (if applicable)

### Using as a Pre-Push Hook (Optional)

**Not required, but helpful.** To automatically run CI checks before pushing:

Create `.git/hooks/pre-push`:

```bash
#!/bin/bash
echo "Running CI checks before push..."
SKIP_BUILD=1 ./scripts/ci-local.sh
```

Then make it executable:

```bash
chmod +x .git/hooks/pre-push
```

You can bypass the hook when needed with:

```bash
git push --no-verify
```

### About act (Docker-based GitHub Actions runner)

There's a tool called [act](https://github.com/nektos/act) that runs GitHub
Actions workflows locally using Docker. However, it requires Docker to be
installed, which isn't available in all development environments. Our bash
scripts provide a simpler, more portable alternative that works anywhere Rust is
installed.

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add tests for new functionality
- Update documentation when adding public APIs

## Core Files

### jeb crate (`crates/jeb/`)

- `src/lib.rs` - Main library with JSON parsing, sorting, merging logic
- `src/main.rs` - CLI implementation
- `src/json_stream.rs` - Stream/text conversion utilities (work in progress)
- `src/sqlite.rs` - SQLite integration with custom functions

### json-encoded-binary crate (`crates/json-encoded-binary/`)

- `src/lib.rs` - JEB85 encoding/decoding implementation with nom parser
