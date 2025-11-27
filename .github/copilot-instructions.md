# GitHub Copilot Instructions for jeb

This file provides guidance to GitHub Copilot when working with code in this repository.

## Project Overview

`jeb` is a Rust workspace for experimenting with machine- and human-sympathetic encoding. The project includes tools for JSON manipulation, binary encoding (JEB85), and related utilities.

## Workspace Structure

This is a Cargo workspace. The current workspace member is:

- `crates/jeb` - Main CLI tool and library for JSON merging, formatting, and searching

Additional crate directories exist in `crates/` but are not currently part of the workspace:

- `crates/slop` - Parser combinators and utilities (experimental)
- `crates/slop-jeb-bin` - Binary encoding utilities (experimental)
- `crates/slop-lenient-json` - Lenient JSON parsing (experimental)
- `crates/slop-wasi-demo` - WASI demonstration (experimental)

## Build, Test, and Lint Commands

When implementing features or fixing bugs, use these commands:

```bash
# Run all tests (debug mode)
cargo test

# Run all tests (release mode)
cargo test --release

# Format code (run before committing)
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run linter (must pass with no warnings)
cargo clippy -- -D warnings

# Build release binary
cargo build --release

# Run all CI checks locally
./scripts/ci-local.sh

# Run CI checks without the slow release build
SKIP_BUILD=1 ./scripts/ci-local.sh
```

## Critical: Version Bumping

**Every pull request MUST bump the version in the root `Cargo.toml`.**

The version format is `0.0.0-vibes-YYYY-MM-DD.xxxx` (date-based versioning).

To generate a new version:

```bash
./scripts/new-version.sh
```

CI will fail if the version is not bumped from the base branch.

## Coding Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes with no warnings
- Add tests for new functionality
- Update documentation when adding public APIs
- Write code comments in English
- Use the Rust 2024 edition features as specified in `Cargo.toml`

## Key Files

### Configuration Files
- `Cargo.toml` - Workspace configuration and dependencies
- `rust-toolchain.toml` - Rust toolchain version
- `rustfmt.toml` - Code formatting configuration
- `clippy.toml` - Linter configuration

### Scripts
- `scripts/test.sh` - Run tests in debug and release mode
- `scripts/clippy.sh` - Run clippy linter
- `scripts/fmt.sh` - Check code formatting
- `scripts/new-version.sh` - Generate new version number
- `scripts/ci-local.sh` - Run all CI checks locally

### Main Source Code
- `crates/jeb/src/lib.rs` - Main library with JSON parsing, sorting, merging logic
- `crates/jeb/src/bin/jeb.rs` - CLI implementation

## Documentation

Key documentation files to reference:

- `crates/jeb/README.md` - Main crate overview
- `crates/slop/CONTRIBUTING.md` - Development workflow and guidelines
- `crates/slop/DESIGN.md` - Architecture and design decisions

## Testing

- Tests are located alongside source files and in `tests/` directories
- Run `cargo test` to execute all tests
- Add tests for new functionality
- Integration tests for the `jeb` binary are in `tests/`

## Dependencies

Workspace dependencies are defined in the root `Cargo.toml`. Key dependencies include:

- `serde` and `serde_json` for JSON serialization
- `clap` for CLI argument parsing
- `tokio` for async runtime
- `indexmap` for ordered maps
- `nom` for parsing

## License

This project is dual-licensed under MIT OR Apache-2.0.
