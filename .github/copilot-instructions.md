# Copilot Instructions for jeb

Note that we are using the nightly-2025-11-28 Rust toolchain for this project
(because we are using some unstable features for formatting and linting — the
actual code should be compatible with stable Rust). Ensure that you're you have
the nightly-2025-11-28 Rust toolchain installed.

## Before Completing Any Task

Always run these commands to ensure your changes are properly formatted, linted,
and tested:

```bash
cargo fix --allow-dirty --allow-staged

cargo clippy --fix --allow-dirty --allow-staged

cargo fmt

cargo test

crates/jeb/examples/all.sh
```

## Code Style

- Don't add comments that explain what the code is doing if the code itself is
  extremely clear and explicit, and using most of the same words as the comment
  would use.
- Do add comments for cases where it's not locally clear why something is being
  done, or it's not clear what's being done, such as to exploit the nuances of
  an algorithm or key invariants.
