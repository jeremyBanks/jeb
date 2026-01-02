# jeb-value specification

This is the [Tracey](https://crates.io/crates/tracey) specification for the
`jeb-value` crate. Run the `tracey` command-line tool to evaluate coverage.

---

r[jeb-value.value.pub-enum]
The crate MUST publicly export the `Value` enum from its root, such that users
might `use json_value::Value`.

r[jeb-value.value.variant-names]
`Value` MUST have variants `Null`, `Boolean`, `Number`, `Bytes`, `String`,
`Array`, `BytesMap`, and `StringMap`. This list is exhaustive.

r[jeb-value.value.variant-types]
Each `Value` variant MUST be a single-element tuple variant over a type of the
same name that's also exported from the crate's root.

r[jeb-value.value.round-trip]
If `Value` defines `From<T>` for any type `T`, `T` MUST also implement
`TryFrom<Value>` which can be used to successfully losslessly round-trip values.

r[jeb-value.variants.round-trip]
If a `Value` variant type `V` defines `From<T>` for any type `T`, or
`TryFrom<T>` and the call to `try_from` returns `Ok(...)`, then `T` MUST also
implement `TryFrom<V>` which can be used to successfully losslessly round-trip
values.

r[jeb-value.value.from-variant]
`Value` MUST implement `From<T>` for each variant type, wrapping it in the
appropriate enum variant.

r[jeb-value.serde.feature]
The crate MUST have a non-default `serde` feature.

r[jeb-value.serde.optional]
The crate SHOULD not depend on the `serde` crate unless the `serde` feature is
enabled.

r[jeb-value.serde.flagged]
Any reference to the `serde` crate MUST be gated behind the `serde` feature
using `cfg!`, `#[cfg ...]`, `#[cfg_attr ...]` or similar.

r[jeb-value.null]
The `Null` variant type MUST be a unit struct.

r[jeb-value.boolean]
The `Boolean` variant type MUST be a single-item tuple struct wrapping a
primitive `bool`.

r[jeb-value.number]
The `Number` variant type MUST be a single-item tuple struct wrapping a
primitive `f64`.


