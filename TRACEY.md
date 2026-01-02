r[workspace.crates.locations]
All Rust crates in this repository SHOULD be located directly under the
`crates/` path.

r[workspace.crates.internal]
Crates that are only meant for internal use _within_ this repository, and are
not currently intended for publication directly or indirectly, SHOULD be named
with a `_` prefix and marked with `publish = false`.

r[workspace.crates.dependency-versions]
Crates SHOULD use workspace dependencies to share version numbers.

r[workspace.crates.no-dependency-features]
Crates MUST NOT use workspace dependencies to share features. (This rule does
not prohibit the use of the `no-default-feature = true` option in the workspace,
which may be required because it's not possible to set to `true` in an
inheriting crate unless it's also `true` in the workspace.)

r[workspace.crates.default-feature]
When it needs to be specified explicitly (i.e. when inheriting from a workspace
dependency definition that was set to `no-default-feature = true` for another
inheritor), the default feature MUST be enabled by using adding `"default"` to
the beginning of the `features` array. This implies that
`no-default-feature = false` MUST never ber used.



