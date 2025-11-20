# Contributing to jeb

## Pull Request Guidelines

### Version Bumping (IMPORTANT!)

**Every pull request MUST bump the version number in `Cargo.toml`.**

Our CI checks enforce that the PR version is different from the base branch
version. If you forget to bump the version, your PR will fail with:

```
ERROR: PR version (X.X.X) must be different from base branch version (X.X.X)
```

To fix this:

1. Run `./scripts/new-version.sh` to generate a new version based on current date/time
2. This will update the version in `Cargo.toml` to format: 0.0.0-vibes-YYYY-MM-DD.xxxx
3. Commit the change

### Before Submitting a PR

1. **Bump the version** in `Cargo.toml` (see above)
2. **Run tests**: `cargo test`
3. **Check formatting**: `cargo fmt --check`
4. **Run linter**: `cargo clippy -- -D warnings`
5. **Build the project**: `cargo build --release`

### Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings

### Running Tests

```bash
# Run all tests
cargo test

# Run tests in release mode
cargo test --release

# Run a specific test
cargo test test_name
```

### Building Documentation

```bash
# Build documentation locally
cargo doc --open

# Build with all features
cargo doc --all-features --open
```
