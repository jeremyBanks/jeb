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

## Testing GitHub Actions Locally

**TL;DR**: Use [act](https://github.com/nektos/act) to run GitHub Actions locally before pushing, catching CI failures early and saving time.

### What is act?

`act` is a tool that runs GitHub Actions workflows on your local machine using Docker. It reads workflows from `.github/workflows/` and executes them in containers that match GitHub's environment, giving you fast feedback before pushing code.

### Why use it?

- **Catch failures early**: Find CI issues locally instead of in GitHub Actions tab
- **Save time**: No need to commit/push/wait to test workflow changes
- **Free CI minutes**: Local runs don't consume GitHub Actions minutes
- **Faster iteration**: Test changes immediately without network round-trips

### Prerequisites

**Docker must be installed and running** - act uses Docker to run workflow containers.

Check if Docker is available:
```bash
docker --version
```

### Installation

Choose one method:

**Linux/macOS (curl script):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**macOS (Homebrew):**
```bash
brew install act
```

**Linux (Nix):**
```bash
nix-env -iA nixpkgs.act
```

**Windows (Chocolatey):**
```bash
choco install act-cli
```

**Other options**: See [nektos/act releases](https://github.com/nektos/act/releases)

### Basic Usage

```bash
# List all workflows and jobs
act -l

# Run all workflows (simulates 'push' event)
act

# Run all workflows for pull_request event
act pull_request

# Run a specific job
act -j test

# Run a specific workflow
act -W .github/workflows/test.yml

# Dry run (show what would run without executing)
act -n
```

### Common Workflows for This Project

```bash
# Run the test workflow (most common - tests, clippy, fmt)
act -j test
act -j clippy
act -j fmt

# Run version check
act -j check-version

# Run everything that would run on a pull request
act pull_request
```

### Tips and Limitations

**First run**: act will prompt you to choose a Docker image size (medium is recommended for Rust projects).

**Secrets**: If workflows need secrets, create `.secrets` file or pass with `-s`:
```bash
act -s GITHUB_TOKEN=your_token
```

**Known limitations**:
- Requires Docker (won't work in environments without it)
- Some GitHub-specific features may not work identically
- Large images can be slow on first download

**Documentation**: Full docs at [nektosact.com](https://nektosact.com)

### Should I use a pre-push hook?

**Optional, not required.** While act can be integrated into git hooks, it's intentionally left as an optional developer tool because:
- Not all developers may have Docker installed
- CI runs are already relatively fast
- Hooks can slow down git operations
- Developers should choose their own workflow

If you want to add it as a personal pre-push hook, create `.git/hooks/pre-push`:
```bash
#!/bin/bash
echo "Running GitHub Actions locally with act..."
act pull_request -q
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
