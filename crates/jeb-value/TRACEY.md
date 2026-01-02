# jeb-value specification

This is the [Tracey](https://crates.io/crates/tracey) specification for the
`jeb-value` crate. Run the `tracey` command-line tool to evaluate coverage.

## Cargo Features

r[jeb-value.dependencies.optional]  
Any optional dependencies SHOULD have a corresponding Cargo feature to enable
them. Multiple optional dependencies MAY be grouped behind a single feature if
they're both required for a single set of functionality.

r[jeb-value.dependencies.limit-internal]  
This crate MUST NOT have any non-test dependencies on other crates within this 
workspace except for `jeb-common` (which MAY be added).

r[jeb-value.dependencies.cfg]  
Any use of an optional dependency MUST be gated behind the corresponding Cargo
feature using `cfg!`, `#[cfg ...]`, `#[cfg_attr ...]` or similar.

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
also implement `TryFrom<Value>` which can losslessly recover any values that
were converted using `From<T>` or a successful `TryFrom<T>`.

r[jeb-value.value.cmp]  
`Value` MUST implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and `Hash`, with
correct non-panicking behavior for all possible values.

r[jeb-value.value.clone]  
`Value` MUST implement `Clone`.

r[jeb-value.value.debug]  
`Value` MUST implement `Debug`.

r[jeb-value.value.static]  
`Value` MUST be `'static`.

r[jeb-value.value.send]  
`Value` MUST be `Send`.

r[jeb-value.value.sync]  
`Value` MUST be `Sync`.

r[jeb-value.value.must-use]  
`Value` MUST be marked `#[must_use]`.

## Variants

r[jeb-value.variants.pub]  
The crate MUST publicly export each variant type from its root (and nowhere
else).

r[jeb-value.variants.tuple]  
Each variant type MUST be a single-item tuple wrapping an inner value.

r[jeb-value.variant.into-value]  
`Value` MUST implement `From<T>` for each variant type, wrapping it in the
appropriate enum variant.

r[jeb-value.variant.round-trip]  
If a variant type `V` defines `From<T>` or `TryFrom<T>` for any type `T`, then
`T` MUST also implement `TryFrom<V>` which can losslessly recover any values
that were converted using `From<T>` or a successful `TryFrom<T>`.

r[jeb-value.variant.try-from]  
Each variant MUST implement `TryFrom<INNER>` for their wrapped inner type. This
may be implicit from a `From<INNER>` implementation or explicit if it's
fallible.

r[jeb-value.variants.into-inner]  
Each variant type MUST implement `into_inner(self): INNER`.

r[jeb-value.variants.as-ref]  
Each variant type MUST implement `AsRef<INNER>`.

r[jeb-value.variants.deref]  
Each variant type MUST implement `Deref<Target=INNER>`.

r[jeb-value.variants.inner-from]  
For each variant type, their inner type MUST implement `From<VARIANT>`.

r[jeb-value.variants.transparent]  
Each variant type MUST be marked `#[repr(transparent)]`.

r[jeb-value.variants.cmp]  
Each variant type MUST implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and
`Hash`, with correct non-panicking behavior for all possible values.

r[jeb-value.variants.cmp.delegate-variants]  
When comparing two `Value` instances containing the same variant type, the 
comparison MUST delegate to the variant type's comparison implementations.

r[jeb-value.variants.cmp.mixed-variants]  
When comparing two `Value` instances of different variant types, MUST NOT be
equal and they MUST follow the order `Null`, `Boolean`, `Number`, `Bytes`,
`String`, `Array`, `BytesMap`, and `StringMap`

r[jeb-value.variants.cmp.delegate-inner]  
When comparing two instances of a variant type, the comparison MUST delegate
to the inner type's comparison implementations unless specified otherwise for
that variant type, for all of `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and
`Hash`.

r[jeb-value.variants.borrow]  
Each variant type MUST implement `Borrow<INNER>` if its comparison and has
implementations simply delegate to the inner type, but MUST NOT implement
`Borrow<INNER>` of it has its own implementation (doing more than
unconditionally delegating) for any of `Eq`, `PartialEq`, `Ord`, `PartialOrd`,
or `Hash`.

r[jeb-value.variants.mut]  
If and only if a variant type implements infallible `From<INNER>` and implements
`Borrow<INNER>`, then it MUST also implement `BorrowMut<INNER>`, `AsMut<INNER>`,
and `DerefMut<Target=INNER>`, otherwise it MUST NOT implement any of those
traits.

r[jeb-value.variants.clone]  
Each variant type MUST implement `Clone`.

r[jeb-value.variants.debug]  
Each variant type MUST implement `Debug`.

r[jeb-value.variants.static]  
Each variant type MUST be `'static`.

r[jeb-value.variants.send]  
Each variant type MUST be `Send`.

r[jeb-value.variants.sync]  
Each variant type MUST be `Sync`.

r[jeb-value.value.must-use]
Each variant MUST be marked `#[must_use]`, except for `Null` which MUST NOT.

### `Null`

r[jeb-value.null]  
The `Null` variant type MUST be a unit struct.

### `Boolean`

r[jeb-value.boolean]  
The `Boolean` variant type MUST be a single-item tuple struct wrapping an
inner primitive `bool`.

### `Number`

r[jeb-value.number]  
The `Number` variant type MUST be a single-item tuple struct wrapping an
inner primitive `f64`.

### `Bytes`

r[jeb-value.bytes]  
The `Bytes` variant type MUST be a single-item tuple struct wrapping an
inner `Vec<u8>`.

### `String`

r[jeb-value.string]  
The `String` variant type MUST be a single-item tuple struct wrapping an
inner `String`.

### `Array`

r[jeb-value.array]  
The `Array` variant type MUST be a single-item tuple struct wrapping an
inner `Vec<Value>`.

### `BytesMap`

r[jeb-value.bytes-map]  
The `BytesMap` variant type MUST be a single-item tuple struct wrapping an
inner `indexmap::IndexMap<Vec<u8>, Value>`.

### `StringMap`

r[jeb-value.string-map]  
The `StringMap` variant type MUST be a single-item tuple struct wrapping an
inner `indexmap::IndexMap<String, Value>`.

## Serde

r[jeb-value.serde.optional]  
Any dependencies on `serde` and other `serde-*` ecosystem crates MUST be
optional, gated behind a `serde` Cargo feature.

r[jeb-value.serde.traits]  
When the `serde` Cargo feature is enabled, `Value` MUST implement
`serde::Serialize` and `serde::Deserialize`.

## Facet

r[jeb-value.facet.optional]  
Any dependencies on `facet` and other `facet-*` ecosystem crates MUST be
optional, gated behind a `facet` Cargo feature.

r[jeb-value.facet.traits]  
When the `facet` Cargo feature is enabled, `Value` and all variant types MUST
implement `Facet`.
