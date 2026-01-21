# Prettyplease Autofix Attempt - Failed

## Goal

Add a `prettyplease`-based autofix to provide comprehensive Rust code formatting via AST parsing/unparsing. The idea was that prettyplease would handle ALL valid Rust syntax since it works at the AST level, providing better coverage than rustfmt.

## Implementation

Successfully implemented:
1. Created `src/prettyplease.rs` module that walks all `*.rs` files
2. Parses each file with `syn::parse_file()`
3. Formats with `prettyplease::unparse()`
4. Integrated into autofix pipeline as first step (before cargo fmt)
5. Created separate `_autofix_prettyplease` binary for isolated testing

## Failure

When testing on the jeb workspace, prettyplease **panicked** on `crates/save/src/bin/save.rs`:

```
thread 'main' panicked at prettyplease-0.2.37/src/item.rs:364:13:
not implemented: Item::Verbatim `use { :: clap :: Parser , :: eyre :: Report , :: save :: cli :: Save } ;`
```

### Root Cause

The file contains:
```rust
use {::clap::Parser, ::eyre::Report, ::save::cli::Save};
```

The leading `::` (Rust 2015 edition absolute path syntax) in grouped use statements causes `syn` to emit an `Item::Verbatim` node, which `prettyplease` explicitly doesn't handle and panics on with "not implemented".

This is valid Rust syntax, but prettyplease cannot handle it.

## Why This Matters

The entire point of using prettyplease was to get **comprehensive** coverage of all Rust syntax. If it can't handle valid Rust code that appears in this codebase, it doesn't serve its purpose.

## Alternatives Considered

1. **Catch panics**: Use `std::panic::catch_unwind` to skip problematic files
   - **Rejected**: Defeats the purpose - we want comprehensive formatting

2. **Fix the source syntax**: Remove `::` prefixes
   - **Not viable**: The syntax is intentionally used (also on line 4: `::color_eyre::install()`)

3. **Upgrade prettyplease**: Try newer versions
   - **Unlikely to help**: This appears to be a fundamental limitation with `Item::Verbatim` handling

4. **File upstream bug**: Report to prettyplease project
   - **Doesn't help us now**: Would need to wait for a fix

## Conclusion

Prettyplease does NOT handle all valid Rust syntax and cannot be relied upon for comprehensive formatting. We're removing it from the autofix pipeline and sticking with `cargo fmt` as the primary Rust formatter.

## Timestamp

Attempted: 2026-01-01
Failed: 2026-01-01
Documented by: Claude Sonnet 4.5
