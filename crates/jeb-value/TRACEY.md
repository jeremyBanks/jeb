# jeb-value specification

This is the [Tracey](https://crates.io/crates/tracey) specification for the
`jeb-value` crate. Run the `tracey` command-line tool to evaluate coverage.

## Cargo Features

r[jeb-value.serde.feature]  
The crate MUST have a non-default `serde` feature.

r[jeb-value.serde.optional]  
The crate SHOULD not depend on the `serde` crate unless the `serde` feature is
enabled.

r[jeb-value.serde.flagged]  
Any reference to the `serde` crate MUST be gated behind the `serde` feature
using `cfg!`, `#[cfg ...]`, `#[cfg_attr ...]` or similar.

## `Value`

r[jeb-value.value.pub]  
The crate MUST publicly export the `Value` enum from its root (and nowhere else),
such that users might `use json_value::Value`.

r[jeb-value.value.enum-variants]  
`Value` MUST be an enum with variants `Null`, `Boolean`, `Number`, `Bytes`,
`String`, `Array`, `BytesMap`, and `StringMap`. This list is exhaustive.

r[jeb-value.value.variant-types]  
Each `Value` enum variant MUST be a single-element tuple variant over a type
(which we'll refer to as a "variant type") of the same name.

r[jeb-value.value.round-trip]  
If `Value` defines `From<T>` or `TryFrom<T>` for any type `T`, then `T` MUST
also implement `TryFrom<Value>` which can losslessly recover any values that was
converted using `From<T>` or a successful `TryFrom<T>`.

r[jeb-value.value.cmp]
`Value` MUST implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and `Hash`, with
correct non-panicking behavior for all possible values.

r[jeb-value.value.clone]
`Value` MUST implement `Clone`.

r[jeb-value.value.debug]
`Value` MUST implement `Debug`.

## Variants

r[jeb-value.variants.pub]
The crate MUST publicly export each variant type from its root (and nowhere
else).

r[jeb-value.variants.tuple]
Each variant type MUST be a single-item tuple wrapping an inner value.

r[jeb-value.variant.into-value]  
`Value` MUST implement `From<T>` for each variant type, wrapping it in the
appropriate enum variant.

r[jeb-value.variant.try-from]
Each variant MUST implement `TryFrom<INNER>` for their wrapper inner type. This
may be implicit from a `From<INNER>` implementation or explicit if it's
fallible.

r[jeb-value.variants.into-inner]
Each variant type MUST provide an `into_inner(self)` implementation which
returns the wrapped inner value.

Each variant type 

r[jeb-value.variants.transparent]
Each variant type MUST be marked `#[repr(transparent)]`.

r[jeb-value.value.cmp]
Each variant type MUST implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and
`Hash`, with correct non-panicking behavior for all possible values.

r[jeb-value.value.clone]
Each variant type MUST implement `Clone`.

r[jeb-value.value.debug]
Each variant type MUST implement `Debug`.




r[jeb-value.variants.round-trip]  
If a variant type `V` defines `From<T>` or `TryFrom<T>` for any type `T`, then
`T` MUST also implement `TryFrom<V>` which can losslessly recover any values
that was converted using `From<T>` or a successful `TryFrom<T>`.

r[jeb-value.variants.to-inner]  
Each 

### `Null`

r[jeb-value.null]  
The `Null` variant type MUST be a unit struct.

r[jeb-value.boolean]  
The `Boolean` variant type MUST be a single-item tuple struct wrapping a
primitive `bool`.

r[jeb-value.number]  
The `Number` variant type MUST be a single-item tuple struct wrapping a
primitive `f64`.


