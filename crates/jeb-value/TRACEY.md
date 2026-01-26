# `jeb-value` specification

This is the [Tracey](https://crates.io/crates/tracey) specification for the
`jeb-value` crate. Run the `tracey` command-line tool to evaluate coverage.

## Code Organization (`jeb-value.src.`)

### Module Structure (`jeb-value.src.modules.`)

r[jeb-value.src.modules.value]
`jeb_value::Value` MUST be defined in `src/value/mod.rs`. That file MUST NOT
directly contain anything except the definition of `jeb_value::Value` and any
item macros (such as derive) that we apply to it, and declarations of
sub-modules. All related code, including inherent impls and trait
implementations, MUST go in sub-modules under `src/value/`.

r[jeb-value.src.modules.variant]
Each variant type MUST be defined in `src/{variant}/mod.rs`, where `{variant}`
is the appropriate snake_case name. That file MUST NOT directly contain anything
except the definition of the variant type and any item macros (such as derive)
that we apply to it, and declarations of sub-modules. All related code,
including inherent impls and trait implementations, MUST go in sub-modules under
`src/{variant}/`.

### Trait Implementation Files (`jeb-value.src.traits.`)

r[jeb-value.src.traits.naming]
Trait implementation files MUST be named using snake_case. Standard library
traits use `{trait}.rs` (e.g., `clone.rs`, `partial_eq.rs`). External crate
traits use `{crate}_{trait}.rs` (e.g., `serde_serialize.rs`).

r[jeb-value.src.traits.placement]
Trait implementations MUST be placed in the submodule of the type which is
`Self`. If `Self` is not a `jeb-value` type, the implementation MUST be placed
in the submodule of the lexicographically-first `jeb-value` type involved (using
spec order: Null, Boolean, Number, Bytes, String, Array, BytesMap, StringMap).

r[jeb-value.src.traits.self-suffix]
When a trait impl file contains implementations where `Self` is NOT the type
that owns the submodule, the file MUST be named with a `_self` suffix (e.g.,
`eq_self.rs` for `impl Eq for OtherType` in the current type's directory).

r[jeb-value.src.traits.into-exception]
Exception to the `_self` suffix rule: files that would be named `from_self.rs`
MUST instead be named `into.rs` (even though they contain `From` impls where
`Self` is not the owning type).

r[jeb-value.src.traits.try-into-exception]
Exception to the `_self` suffix rule: files that would be named
`try_from_self.rs` MUST instead be named `try_into.rs` (even though they contain
`TryFrom` impls where `Self` is not the owning type).

### Type Ordering (`jeb-value.src.ordering.`)

r[jeb-value.src.ordering.spec]
When lexicographic ordering among variant types is needed for any decision
(including file placement and comparison ordering), the spec order MUST be used:
Null, Boolean, Number, Bytes, String, Array, BytesMap, StringMap.

## Conversions (`jeb-value.conv.`)

### Foundation (`jeb-value.conv.foundation.`)

r[jeb-value.conv.foundation.from-tryfrom]
All type conversions MUST be implemented via `From<T>` or `TryFrom<T>` traits as
the foundation. Other conversion mechanisms MUST delegate to these traits.

r[jeb-value.conv.foundation.deref]
`Deref<Target=INNER>` MUST return `&self.0` (foundational for borrowed access).

r[jeb-value.conv.foundation.deref-mut]
`DerefMut<Target=INNER>` MUST return `&mut self.0`.

r[jeb-value.conv.foundation.as-ref]
`AsRef<INNER>` MUST delegate to `Deref::deref()`.

r[jeb-value.conv.foundation.as-mut]
`AsMut<INNER>` MUST delegate to `DerefMut::deref_mut()`.

r[jeb-value.conv.foundation.borrow]
`Borrow<INNER>` MUST delegate to `Deref::deref()` (when allowed per borrow
rules).

r[jeb-value.conv.foundation.borrow-mut]
`BorrowMut<INNER>` MUST delegate to `DerefMut::deref_mut()`.

r[jeb-value.conv.foundation.into-inner]
`.into_inner(self) -> INNER` MUST use `INNER::from(self)` (delegates to `From`).

r[jeb-value.conv.foundation.to-inner]
`.to_inner(&self) -> INNER` MUST use `INNER::from(self.as_inner().clone())` or
equivalent cloning delegation.

r[jeb-value.conv.foundation.as-inner]
`.as_inner(&self) -> &INNER` MUST use `self.deref()` (delegates to `Deref`).

### Round-Trip (`jeb-value.conv.round-trip.`)

r[jeb-value.conv.round-trip.value]
If `jeb_value::Value` defines `From<T>` or `TryFrom<T>` for any type `T`, then
`T` MUST also implement `TryFrom<jeb_value::Value>` which can losslessly recover
any values that were converted using `From<T>` or a successful `TryFrom<T>`.
This is a universal requirement applying to all such conversions, regardless of
whether the other type is internal, external, or built-in.

r[jeb-value.conv.round-trip.variant]
If a variant type `V` defines `From<T>` or `TryFrom<T>` for any type `T`, then
`T` MUST also implement `TryFrom<V>` which can losslessly recover any values
that were converted using `From<T>` or a successful `TryFrom<T>`. This is a
universal requirement applying to all such conversions, regardless of whether
the other type is internal, external, or built-in.

### Numeric Exactness (`jeb-value.conv.numeric.`)

r[jeb-value.conv.numeric.exactness]
`jeb_value::Number` MUST accept only values that are numerically identical to
their f64 representation. For integers, a value is acceptable if
`(value as f64) as T == value` AND the conversion is numerically identical (not
just round-trip equal). Large integers outside the "safe integer range" CAN be
accepted if they are exactly representable in f64 (e.g., `1u64 << 54` is
acceptable because it equals 2^54 exactly, but `(1u64 << 54) + 1` is NOT
acceptable because precision is lost).

r[jeb-value.conv.numeric.f32]
`jeb_value::Number` MUST always accept finite f32 values, as f32 fits perfectly
within f64's mantissa and all finite f32 values are exactly representable.

r[jeb-value.conv.numeric.bit-preservation]
For lossless round-tripping, f32 NaN bit patterns MUST be preserved exactly (NO
canonicalization). The sign bit of `-0.0` vs `+0.0` MUST also be preserved.

### Fallback Behavior (`jeb-value.conv.fallback.`)

r[jeb-value.conv.fallback.numeric]
`jeb_value::Value::try_from(numeric)` MUST try `jeb_value::Number` first, and
fall back to `jeb_value::Bytes` if the value is not exactly representable in
f64. This applies to ALL numeric types: f32, f64, i8-i128, u8-u128.

r[jeb-value.conv.fallback.bytes-encoding]
When falling back to `jeb_value::Bytes` for a numeric value, the encoding MUST
be the big-endian byte representation of the original value.

r[jeb-value.conv.fallback.infallible]
Due to the fallback behavior, `jeb_value::Value::try_from()` MUST be infallible
for all numeric types (returning `Result<Value, Infallible>` or implementing
`From` directly).

### Lifetime-Preserving Conversions (`jeb-value.conv.lifetime.`)

r[jeb-value.conv.lifetime.borrowed]
For types that support borrowed access (like `jeb_value::Bytes` and
`jeb_value::String`), lifetime-preserving `TryFrom` implementations MUST be
provided using the pattern `impl<'a> TryFrom<&'a jeb_value::Value> for &'a [u8]`
and similar. This enables generic map access methods to accept both native key
types and `jeb_value::Value`.

## Cargo Features (`jeb-value.features.`)

### Core (`jeb-value.features.core.`)

r[jeb-value.features.core.optional]
Any optional dependency MUST have a correspondingly-named Cargo feature to
enable it. If an optional dependency is known to depend on another optional
dependency, their corresponding features MUST also have the same dependency
relationship.

r[jeb-value.features.core.grouped]
Multiple optional dependencies MAY be grouped behind a single feature if they're
both required for a single set of functionality.

r[jeb-value.features.core.cfg]
Any use of an optional dependency MUST be gated behind the corresponding Cargo
feature using `cfg!`, `#[cfg ...]`, `#[cfg_attr ...]` or similar.

r[jeb-value.features.core.limit-internal]
This crate MUST NOT have any non-test dependencies on other crates within this
workspace except for `jeb-common` (which MAY be added).

r[jeb-value.features.core.language]
Rules whose behavior depends on optional crates MUST state "when the `X` feature
is enabled" to make dependencies explicit in the specification.

### Serde (`jeb-value.features.serde.`)

r[jeb-value.features.serde.optional]
Any dependencies on `serde` and other `serde-*` ecosystem crates MUST be
optional, gated behind a `serde` Cargo feature.

### Serde JSON (`jeb-value.features.serde-json.`)

r[jeb-value.features.serde-json.depends]
When the `serde-json` feature is enabled, it MUST depend on the `serde` feature
and MUST depend on the `serde_json` crate.

r[jeb-value.features.serde-json.from]
When the `serde-json` feature is enabled, `jeb_value::Value` MUST implement
`Self::from_serde_json_value(value: serde_json::Value) -> Self`.

r[jeb-value.features.serde-json.to]
When the `serde-json` feature is enabled, `jeb_value::Value` MUST implement
`Self::to_serde_json_value(&self) -> serde_json::Value`.

r[jeb-value.features.serde-json.distinct]
The `from_serde_json_value` and `to_serde_json_value` methods are DIFFERENT from
`from_serde`/`to_serde` (which use serde traits generically). The serde_json
methods operate directly on `serde_json::Value` without going through serde
serialization/deserialization.

### Facet (`jeb-value.features.facet.`)

r[jeb-value.features.facet.optional]
Any dependencies on `facet` and other `facet-*` ecosystem crates MUST be
optional, gated behind a `facet` Cargo feature.

### Facet Value (`jeb-value.features.facet-value.`)

r[jeb-value.features.facet-value.depends]
When the `facet-value` feature is enabled, it MUST depend on the `facet` feature
and MUST depend on the `facet-value` crate.

r[jeb-value.features.facet-value.from]
When the `facet-value` feature is enabled, `jeb_value::Value` MUST implement
`Self::from_facet_value(value: facet_value::Value) -> Self`.

r[jeb-value.features.facet-value.to]
When the `facet-value` feature is enabled, `jeb_value::Value` MUST implement
`Self::to_facet_value(&self) -> facet_value::Value`.

r[jeb-value.features.facet-value.distinct]
The `from_facet_value` and `to_facet_value` methods are DIFFERENT from
`from_facet`/`to_facet` (which use the Facet trait). The facet_value methods
operate directly on `facet_value::Value` without going through the Facet trait.

## Value (`jeb-value.value.`)

### Definition (`jeb-value.value.def.`)

r[jeb-value.value.def.pub]
The crate MUST publicly export `jeb_value::Value` from its root (and nowhere
else), such that users might `use jeb_value::Value`.

r[jeb-value.value.def.enum-variants]
`jeb_value::Value` MUST be an enum with variants `Null`, `Boolean`, `Number`,
`Bytes`, `String`, `Array`, `BytesMap`, and `StringMap`. This list is
exhaustive.

r[jeb-value.value.def.variant-types]
Each `jeb_value::Value` enum variant MUST be a single-element tuple variant over
a type (which we'll refer to as a "variant type") of the same name.

### Traits (`jeb-value.value.traits.`)

r[jeb-value.value.traits.eq]
`jeb_value::Value` MUST implement `Eq` with correct non-panicking behavior for
all possible values.

r[jeb-value.value.traits.partial-eq]
`jeb_value::Value` MUST implement `PartialEq` with correct non-panicking
behavior for all possible values.

r[jeb-value.value.traits.ord]
`jeb_value::Value` MUST implement `Ord` with correct non-panicking behavior for
all possible values.

r[jeb-value.value.traits.partial-ord]
`jeb_value::Value` MUST implement `PartialOrd` with correct non-panicking
behavior for all possible values.

r[jeb-value.value.traits.hash]
`jeb_value::Value` MUST implement `Hash` with correct non-panicking behavior for
all possible values.

r[jeb-value.value.traits.clone]
`jeb_value::Value` MUST implement `Clone`.

r[jeb-value.value.traits.debug]
`jeb_value::Value` MUST implement `Debug`.

r[jeb-value.value.traits.static]
`jeb_value::Value` MUST be `'static`.

r[jeb-value.value.traits.send]
`jeb_value::Value` MUST be `Send`.

r[jeb-value.value.traits.sync]
`jeb_value::Value` MUST be `Sync`.

r[jeb-value.value.traits.must-use]
`jeb_value::Value` MUST be marked `#[must_use]`.

### Variant Accessors (`jeb-value.value.accessors.`)

r[jeb-value.value.accessors.from]
`jeb_value::Value` MUST implement `From<T>` for each variant type, wrapping it
in the appropriate enum variant.

r[jeb-value.value.accessors.as]
`jeb_value::Value` MUST implement an `as_{variant}(&self) -> Option<&{Variant}>`
method for each variant type.

r[jeb-value.value.accessors.to]
`jeb_value::Value` MUST implement a `to_{variant}(&self) -> Option<{Variant}>`
method for each variant type.

r[jeb-value.value.accessors.into]
`jeb_value::Value` MUST implement an `into_{variant}(self) -> Option<{Variant}>`
method for each variant type.

r[jeb-value.value.accessors.unwrap]
`jeb_value::Value` MUST implement an `unwrap_{variant}(self) -> {Variant}`
method for each variant type, which panics if the `jeb_value::Value` is not of
the expected variant type.

## Variant Types (`jeb-value.variant.`)

### Common Requirements (`jeb-value.variant.common.`)

r[jeb-value.variant.common.pub]
The crate MUST publicly export each variant type from its root (and nowhere
else).

r[jeb-value.variant.common.tuple]
Each variant type MUST be a single-item tuple wrapping an inner value.

r[jeb-value.variant.common.try-from-inner]
Each variant MUST implement `TryFrom<INNER>` for their wrapped inner type. This
may be implicit from a `From<INNER>` implementation or explicit if it's
fallible.

r[jeb-value.variant.common.try-from-other-via-inner]
Given two variant types `V` and `W`, and their respective inner types `INNER_V`
and `INNER_W`, then if `V` implements `From<INNER_W>` then `V` MUST also
implement `From<W>` by unwrapping `W` to get its inner type and then using the
`From<INNER_W>` implementation to convert it to `V`. Otherwise, if `V`
implements `TryFrom<INNER_W>`, then `V` MUST also implement `TryFrom<W>` by
unwrapping `W` to get its inner type and then using the `TryFrom<INNER_W>`
implementation to convert it to `V`.

r[jeb-value.variant.common.constructor]
If a variant implements infallible `From<INNER>`, it MUST also provide a public
`new(inner: INNER) -> Self` constructor function.

r[jeb-value.variant.common.as-ref]
Each variant type MUST implement `AsRef<INNER>`.

r[jeb-value.variant.common.deref]
Each variant type MUST implement `Deref<Target=INNER>`.

r[jeb-value.variant.common.inner-from]
For each variant type, their inner type MUST implement `From<{Variant}>`.

r[jeb-value.variant.common.into-inner]
Each variant type MUST implement `into_inner(self) -> INNER`.

r[jeb-value.variant.common.to-inner]
Each variant type MUST implement `to_inner(&self) -> INNER`.

r[jeb-value.variant.common.as-inner]
Each variant type MUST implement `as_inner(&self) -> &INNER`.

r[jeb-value.variant.common.into-named-inner]
Each variant type MUST implement `into_{inner}(self) -> INNER`, where `{inner}`
is the appropriately-formatted inner type name (ignoring any generic
parameters).

r[jeb-value.variant.common.to-named-inner]
Each variant type MUST implement `to_{inner}(&self) -> INNER`, where `{inner}`
is the appropriately-formatted inner type name (ignoring any generic
parameters).

r[jeb-value.variant.common.as-named-inner]
Each variant type MUST implement `as_{inner}(&self) -> &INNER`, where `{inner}`
is the appropriately-formatted inner type name (ignoring any generic
parameters).

r[jeb-value.variant.common.transparent]
Each variant type MUST be marked `#[repr(transparent)]`.

r[jeb-value.variant.common.eq]
Each variant type MUST implement `Eq` with correct non-panicking behavior for
all possible values.

r[jeb-value.variant.common.partial-eq]
Each variant type MUST implement `PartialEq` with correct non-panicking behavior
for all possible values.

r[jeb-value.variant.common.ord]
Each variant type MUST implement `Ord` with correct non-panicking behavior for
all possible values.

r[jeb-value.variant.common.partial-ord]
Each variant type MUST implement `PartialOrd` with correct non-panicking
behavior for all possible values.

r[jeb-value.variant.common.hash]
Each variant type MUST implement `Hash` with correct non-panicking behavior for
all possible values.

r[jeb-value.variant.common.cmp-delegate-variants]
When comparing two `jeb_value::Value` instances containing the same variant
type, the comparison MUST delegate to the variant type's comparison
implementations.

r[jeb-value.variant.common.cmp-mixed-variants]
When comparing two `jeb_value::Value` instances of different variant types, they
MUST NOT be equal and MUST follow the order: Null, Boolean, Number, Bytes,
String, Array, BytesMap, StringMap.

r[jeb-value.variant.common.eq-delegate-inner]
When comparing two instances of a variant type for equality, `Eq` MUST delegate
to the inner type's `Eq` implementation unless specified otherwise for that
variant type.

r[jeb-value.variant.common.partial-eq-delegate-inner]
When comparing two instances of a variant type for equality, `PartialEq` MUST
delegate to the inner type's `PartialEq` implementation unless specified
otherwise for that variant type.

r[jeb-value.variant.common.ord-delegate-inner]
When comparing two instances of a variant type for ordering, `Ord` MUST delegate
to the inner type's `Ord` implementation unless specified otherwise for that
variant type.

r[jeb-value.variant.common.partial-ord-delegate-inner]
When comparing two instances of a variant type for ordering, `PartialOrd` MUST
delegate to the inner type's `PartialOrd` implementation unless specified
otherwise for that variant type.

r[jeb-value.variant.common.hash-delegate-inner]
When hashing a variant type, `Hash` MUST delegate to the inner type's `Hash`
implementation unless specified otherwise for that variant type.

r[jeb-value.variant.common.borrow]
Each variant type MUST implement `Borrow<INNER>` if its comparison
implementations simply delegate to the inner type, but MUST NOT implement
`Borrow<INNER>` if it has its own implementation (doing more than
unconditionally delegating) for any of `Eq`, `PartialEq`, `Ord`, `PartialOrd`,
or `Hash`.

r[jeb-value.variant.common.mut]
If and only if a variant type implements infallible `From<INNER>` and implements
`Borrow<INNER>`, then it MUST also implement `BorrowMut<INNER>`, `AsMut<INNER>`,
and `DerefMut<Target=INNER>`, otherwise it MUST NOT implement any of those
traits.

r[jeb-value.variant.common.clone]
Each variant type MUST implement `Clone`.

r[jeb-value.variant.common.debug]
Each variant type MUST implement `Debug`.

r[jeb-value.variant.common.static]
Each variant type MUST be `'static`.

r[jeb-value.variant.common.send]
Each variant type MUST be `Send`.

r[jeb-value.variant.common.sync]
Each variant type MUST be `Sync`.

r[jeb-value.variant.common.must-use]
Each variant type MUST be marked `#[must_use]` unless specified otherwise for
that variant type.

## Null (`jeb-value.null.`)

r[jeb-value.null.struct]
`jeb_value::Null` MUST be a single-item tuple struct wrapping an inner primitive
unit value `()`.

r[jeb-value.null.must-use]
`jeb_value::Null` MUST NOT be marked `#[must_use]`.

r[jeb-value.null.from-inner]
`jeb_value::Null` MUST implement `From<()>`.

r[jeb-value.null.try-from-bool]
`jeb_value::Null` MUST implement `TryFrom<bool>`, accepting only `false`.

r[jeb-value.null.try-from-f32]
`jeb_value::Null` MUST implement `TryFrom<f32>`, accepting only `+0.0`.

r[jeb-value.null.try-from-f64]
`jeb_value::Null` MUST implement `TryFrom<f64>`, accepting only `+0.0`.

r[jeb-value.null.try-from-i8]
`jeb_value::Null` MUST implement `TryFrom<i8>`, accepting only `0`.

r[jeb-value.null.try-from-i16]
`jeb_value::Null` MUST implement `TryFrom<i16>`, accepting only `0`.

r[jeb-value.null.try-from-i32]
`jeb_value::Null` MUST implement `TryFrom<i32>`, accepting only `0`.

r[jeb-value.null.try-from-i64]
`jeb_value::Null` MUST implement `TryFrom<i64>`, accepting only `0`.

r[jeb-value.null.try-from-i128]
`jeb_value::Null` MUST implement `TryFrom<i128>`, accepting only `0`.

r[jeb-value.null.try-from-u8]
`jeb_value::Null` MUST implement `TryFrom<u8>`, accepting only `0`.

r[jeb-value.null.try-from-u16]
`jeb_value::Null` MUST implement `TryFrom<u16>`, accepting only `0`.

r[jeb-value.null.try-from-u32]
`jeb_value::Null` MUST implement `TryFrom<u32>`, accepting only `0`.

r[jeb-value.null.try-from-u64]
`jeb_value::Null` MUST implement `TryFrom<u64>`, accepting only `0`.

r[jeb-value.null.try-from-u128]
`jeb_value::Null` MUST implement `TryFrom<u128>`, accepting only `0`.

r[jeb-value.null.try-from-vec]
`jeb_value::Null` MUST implement `TryFrom<Vec<T>>` (where T is _unconstrained_)
with only the empty vector being accepted.

r[jeb-value.null.try-from-ordermap]
`jeb_value::Null` MUST implement `TryFrom<ordermap<K, V>>` (where K and V are
_unconstrained_) with only the empty map being accepted.

r[jeb-value.null.into-unit]
The unit type `()` MUST implement `From<jeb_value::Null>`.

r[jeb-value.null.into-bool]
`bool` MUST implement `From<jeb_value::Null>`, mapping to `false`.

r[jeb-value.null.into-f32]
`f32` MUST implement `From<jeb_value::Null>`, mapping to `0.0`.

r[jeb-value.null.into-f64]
`f64` MUST implement `From<jeb_value::Null>`, mapping to `0.0`.

r[jeb-value.null.into-i8]
`i8` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-i16]
`i16` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-i32]
`i32` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-i64]
`i64` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-i128]
`i128` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-u8]
`u8` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-u16]
`u16` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-u32]
`u32` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-u64]
`u64` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-u128]
`u128` MUST implement `From<jeb_value::Null>`, mapping to `0`.

r[jeb-value.null.into-vec]
`Vec<T>` (where T is unconstrained) MUST implement `From<jeb_value::Null>`,
mapping to an empty vector.

r[jeb-value.null.into-ordermap]
`ordermap<K, V>` (where K and V are unconstrained) MUST implement
`From<jeb_value::Null>`, mapping to an empty map.

## Boolean (`jeb-value.boolean.`)

r[jeb-value.boolean.struct]
`jeb_value::Boolean` MUST be a single-item tuple struct wrapping an inner
primitive `bool`.

r[jeb-value.boolean.from-inner]
`jeb_value::Boolean` MUST implement `From<bool>`.

r[jeb-value.boolean.from-false]
`jeb_value::Boolean` MUST implement `From<()>` (mapping to `false`).

r[jeb-value.boolean.try-from-f32]
`jeb_value::Boolean` MUST implement `TryFrom<f32>`, with `+0.0` mapping to
false, `1.0` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-f64]
`jeb_value::Boolean` MUST implement `TryFrom<f64>`, with `+0.0` mapping to
false, `1.0` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-i8]
`jeb_value::Boolean` MUST implement `TryFrom<i8>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-i16]
`jeb_value::Boolean` MUST implement `TryFrom<i16>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-i32]
`jeb_value::Boolean` MUST implement `TryFrom<i32>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-i64]
`jeb_value::Boolean` MUST implement `TryFrom<i64>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-i128]
`jeb_value::Boolean` MUST implement `TryFrom<i128>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-u8]
`jeb_value::Boolean` MUST implement `TryFrom<u8>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-u16]
`jeb_value::Boolean` MUST implement `TryFrom<u16>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-u32]
`jeb_value::Boolean` MUST implement `TryFrom<u32>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-u64]
`jeb_value::Boolean` MUST implement `TryFrom<u64>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

r[jeb-value.boolean.try-from-u128]
`jeb_value::Boolean` MUST implement `TryFrom<u128>`, with `0` mapping to false,
`1` mapping to true, and all other values being rejected.

## Number (`jeb-value.number.`)

r[jeb-value.number.struct]
`jeb_value::Number` MUST be a single-item tuple struct wrapping an inner
primitive `f64`.

r[jeb-value.number.finite]
`jeb_value::Number` MUST only be constructible with finite `f64` values
(excluding NaN or Infinity, including `-0`). By enforcing this everywhere that a
value can be constructed, all operations on `jeb_value::Number` can safely
assume the inner value is always finite without needing to re-validate it.

r[jeb-value.number.constructor]
`jeb_value::Number` MUST provide a public `new(value: f64) -> Option<Self>`
constructor function which returns `Some` if the provided `f64` is finite, and
`None` otherwise.

r[jeb-value.number.try-from-inner]
`jeb_value::Number` MUST implement `TryFrom<f64>`, returning an error if the
provided `f64` is not finite.

r[jeb-value.number.no-from-inner]
`jeb_value::Number` MUST NOT implement `From<f64>`.

r[jeb-value.number.cmp-no-delegate]
`jeb_value::Number`'s implementations of `Eq`, `PartialEq`, `Ord`, `PartialOrd`,
and `Hash` MUST NOT delegate directly to the inner `f64` type's trait
implementations.

r[jeb-value.number.eq-total-cmp]
`jeb_value::Number`'s `Eq` implementation MUST delegate comparison to
`f64::total_cmp`. This ensures a total ordering for all possible `f64` values.

r[jeb-value.number.partial-eq-total-cmp]
`jeb_value::Number`'s `PartialEq` implementation MUST delegate comparison to
`f64::total_cmp`. This ensures a total ordering for all possible `f64` values.

r[jeb-value.number.ord-total-cmp]
`jeb_value::Number`'s `Ord` implementation MUST delegate comparison to
`f64::total_cmp`. This ensures a total ordering for all possible `f64` values.

r[jeb-value.number.partial-ord-total-cmp]
`jeb_value::Number`'s `PartialOrd` implementation MUST delegate comparison to
`f64::total_cmp`. This ensures a total ordering for all possible `f64` values.

r[jeb-value.number.hash-to-be-bytes]
`jeb_value::Number`'s `Hash` implementation MUST delegate to the result of
`.to_be_bytes()`. This ensures distinct hashing for all possible `f64` values.

## Bytes (`jeb-value.bytes.`)

r[jeb-value.bytes.struct]
`jeb_value::Bytes` MUST be a single-item tuple struct wrapping an inner
`Vec<u8>`.

r[jeb-value.bytes.from-inner]
`jeb_value::Bytes` MUST implement `From<Vec<u8>>`.

r[jeb-value.bytes.from-slice]
`jeb_value::Bytes` MUST implement `From<&[u8]>`.

r[jeb-value.bytes.from-iterator]
`jeb_value::Bytes` MUST implement `FromIterator<u8>`.

r[jeb-value.bytes.from-slice-iterator]
`jeb_value::Bytes` MUST implement `FromIterator<&[u8]>`.

r[jeb-value.bytes.from-unit]
`jeb_value::Bytes` MUST implement `From<()>`, mapping the unit type to an empty
byte string.

r[jeb-value.bytes.from-bool]
`jeb_value::Bytes` MUST implement `From<bool>`, with `false` mapping to a single
zero byte and `true` mapping to a single byte with value `0x01`.

r[jeb-value.bytes.from-char]
`jeb_value::Bytes` MUST implement `From<char>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-f32]
`jeb_value::Bytes` MUST implement `From<f32>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-f64]
`jeb_value::Bytes` MUST implement `From<f64>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-i8]
`jeb_value::Bytes` MUST implement `From<i8>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-i16]
`jeb_value::Bytes` MUST implement `From<i16>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-i32]
`jeb_value::Bytes` MUST implement `From<i32>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-i64]
`jeb_value::Bytes` MUST implement `From<i64>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-i128]
`jeb_value::Bytes` MUST implement `From<i128>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-u8]
`jeb_value::Bytes` MUST implement `From<u8>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-u16]
`jeb_value::Bytes` MUST implement `From<u16>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-u32]
`jeb_value::Bytes` MUST implement `From<u32>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-u64]
`jeb_value::Bytes` MUST implement `From<u64>`, mapping to its big-endian byte
representation.

r[jeb-value.bytes.from-u128]
`jeb_value::Bytes` MUST implement `From<u128>`, mapping to its big-endian byte
representation.

### Bytes-String Bijective Encoding (`jeb-value.bytes.encoding.`)

r[jeb-value.bytes.encoding.overview]
`jeb_value::Bytes` and `jeb_value::String` MUST support a bijective encoding
scheme using `\b` (0x08, backspace) as the sole escape character. This enables
lossless round-tripping between bytes and strings.

r[jeb-value.bytes.encoding.bytes-to-string]
`jeb_value::Bytes` to `jeb_value::String` encoding MUST follow these rules:
1. If valid UTF-8 and doesn't start with `\b`: literal string (no prefix)
2. If valid UTF-8 and starts with `\b`: prepend `\b` (becomes `\b\b...`)
3. If not valid UTF-8: `\b` + bytes_as_text(bytes)

r[jeb-value.bytes.encoding.string-to-bytes]
`jeb_value::String` to `jeb_value::Bytes` decoding MUST follow these rules:
1. If starts with `\b\b`: strip ONE `\b`, rest is literal UTF-8 bytes
2. If starts with `\b` + [non-`\b`]: decode rest via bytes_as_text
3. Otherwise: literal UTF-8 bytes

r[jeb-value.bytes.encoding.bytes-as-text-constraint]
The `jeb_bytes_as_text` encoding used for non-UTF-8 bytes MUST have an output
alphabet that does NOT include `\b` (0x08). Base64 (`[A-Za-z0-9+/=]`) satisfies
this constraint.

r[jeb-value.bytes.encoding.bijective]
The encoding MUST be bijective: every byte sequence maps to exactly one string,
and every string maps to exactly one byte sequence. The three categories of
output are disjoint because bytes_as_text output cannot start with `\b`.

## String (`jeb-value.string.`)

r[jeb-value.string.struct]
`jeb_value::String` MUST be a single-item tuple struct wrapping an inner
`String`.

r[jeb-value.string.from-inner]
`jeb_value::String` MUST implement `From<String>`.

r[jeb-value.string.from-str]
`jeb_value::String` MUST implement `From<&str>`.

r[jeb-value.string.from-char-iterator]
`jeb_value::String` MUST implement `FromIterator<char>`.

r[jeb-value.string.from-string-iterator]
`jeb_value::String` MUST implement `FromIterator<String>`.

r[jeb-value.string.from-str-iterator]
`jeb_value::String` MUST implement `FromIterator<&str>`.

## Array (`jeb-value.array.`)

r[jeb-value.array.struct]
`jeb_value::Array` MUST be a single-item tuple struct wrapping an inner
`Vec<jeb_value::Value>`.

r[jeb-value.array.len]
`jeb_value::Array` MUST implement `.len(&self) -> usize`.

r[jeb-value.array.is-empty]
`jeb_value::Array` MUST implement `.is_empty(&self) -> bool`.

r[jeb-value.array.iter]
`jeb_value::Array` MUST implement
`.iter(&self) -> impl Iterator<Item=&jeb_value::Value>`.

r[jeb-value.array.into-iterator]
`jeb_value::Array` MUST implement `IntoIterator<Item=jeb_value::Value>`.

r[jeb-value.array.from-inner]
`jeb_value::Array` MUST implement `From<Vec<jeb_value::Value>>`.

r[jeb-value.array.from-slice]
`jeb_value::Array` MUST implement `From<&[jeb_value::Value]>`.

r[jeb-value.array.from-iterator]
`jeb_value::Array` MUST implement `FromIterator<jeb_value::Value>`.

## BytesMap (`jeb-value.bytes-map.`)

r[jeb-value.bytes-map.struct]
`jeb_value::BytesMap` MUST be a single-item tuple struct wrapping an inner
`ordermap::ordermap<Vec<u8>, jeb_value::Value>`.

r[jeb-value.bytes-map.len]
`jeb_value::BytesMap` MUST implement `.len(&self) -> usize`.

r[jeb-value.bytes-map.is-empty]
`jeb_value::BytesMap` MUST implement `.is_empty(&self) -> bool`.

r[jeb-value.bytes-map.generic-access]
`jeb_value::BytesMap` MUST use lifetime-preserving `TryFrom` in generic bounds
for access methods, enabling both native key types (`&[u8]`) AND
`&jeb_value::Value` to work:
```rust
fn get<'a, Q>(&self, key: &'a Q) -> Option<&jeb_value::Value>
where
    &'a [u8]: TryFrom<&'a Q>
```

r[jeb-value.bytes-map.index]
`jeb_value::BytesMap` MUST implement `Index` using the generic access pattern
described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.bytes-map.get]
`jeb_value::BytesMap` MUST implement `.get()` using the generic access pattern
described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.bytes-map.contains-key]
`jeb_value::BytesMap` MUST implement `.contains_key()` using the generic access
pattern described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.bytes-map.from-inner]
`jeb_value::BytesMap` MUST implement
`From<ordermap::ordermap<Vec<u8>, jeb_value::Value>>`.

r[jeb-value.bytes-map.keys]
`jeb_value::BytesMap` MUST implement
`.keys(&self) -> impl Iterator<Item=&Vec<u8>>`.

r[jeb-value.bytes-map.values]
`jeb_value::BytesMap` MUST implement
`.values(&self) -> impl Iterator<Item=&jeb_value::Value>`.

r[jeb-value.bytes-map.iter]
`jeb_value::BytesMap` MUST implement
`.iter(&self) -> impl Iterator<Item=(&Vec<u8>, &jeb_value::Value)>`.

r[jeb-value.bytes-map.into-iterator]
`jeb_value::BytesMap` MUST implement
`IntoIterator<Item=(Vec<u8>, jeb_value::Value)>`.

## StringMap (`jeb-value.string-map.`)

r[jeb-value.string-map.struct]
`jeb_value::StringMap` MUST be a single-item tuple struct wrapping an inner
`ordermap::ordermap<String, jeb_value::Value>`.

r[jeb-value.string-map.len]
`jeb_value::StringMap` MUST implement `.len(&self) -> usize`.

r[jeb-value.string-map.is-empty]
`jeb_value::StringMap` MUST implement `.is_empty(&self) -> bool`.

r[jeb-value.string-map.generic-access]
`jeb_value::StringMap` MUST use lifetime-preserving `TryFrom` in generic bounds
for access methods, enabling both native key types (`&str`) AND
`&jeb_value::Value` to work:
```rust
fn get<'a, Q>(&self, key: &'a Q) -> Option<&jeb_value::Value>
where
    &'a str: TryFrom<&'a Q>
```

r[jeb-value.string-map.index]
`jeb_value::StringMap` MUST implement `Index` using the generic access pattern
described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.string-map.get]
`jeb_value::StringMap` MUST implement `.get()` using the generic access pattern
described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.string-map.contains-key]
`jeb_value::StringMap` MUST implement `.contains_key()` using the generic access
pattern described above, accepting both native key types and `jeb_value::Value`.

r[jeb-value.string-map.from-inner]
`jeb_value::StringMap` MUST implement
`From<ordermap::ordermap<String, jeb_value::Value>>`.

r[jeb-value.string-map.keys]
`jeb_value::StringMap` MUST implement
`.keys(&self) -> impl Iterator<Item=&String>`.

r[jeb-value.string-map.values]
`jeb_value::StringMap` MUST implement
`.values(&self) -> impl Iterator<Item=&jeb_value::Value>`.

r[jeb-value.string-map.iter]
`jeb_value::StringMap` MUST implement
`.iter(&self) -> impl Iterator<Item=(&String, &jeb_value::Value)>`.

r[jeb-value.string-map.into-iterator]
`jeb_value::StringMap` MUST implement
`IntoIterator<Item=(String, jeb_value::Value)>`.

## Serde (`jeb-value.serde.`)

### Core (`jeb-value.serde.core.`)

r[jeb-value.serde.core.serialize]
When the `serde` feature is enabled, `jeb_value::Value` MUST implement
`serde::Serialize`.

r[jeb-value.serde.core.deserialize]
When the `serde` feature is enabled, `jeb_value::Value` MUST implement
`serde::Deserialize`.

r[jeb-value.serde.core.representation]
When the `serde` feature is enabled, `jeb_value::Value`'s implementations of
`serde::Serialize` and `serde::Deserialize` MUST be compatible with the
(default) externally-tagged enum representation.

r[jeb-value.serde.core.serializer]
When the `serde` feature is enabled, the crate MUST provide a `ValueSerializer`
type implementing `serde::ser::Serializer` which serializes an arbitrary
serde-serializable type into a `jeb_value::Value`. This MUST NOT use our own
`jeb_value::Value`'s `serde::Serialize` implementation, as the behavior will
differ. This type MUST be publicly exported from `crate::serde`.

r[jeb-value.serde.core.deserializer]
When the `serde` feature is enabled, the crate MUST provide a `ValueDeserializer`
type implementing `serde::de::Deserializer` which deserializes an arbitrary
serde-deserializable type from a `jeb_value::Value`. This MUST NOT use our own
`jeb_value::Value`'s `serde::Deserialize` implementation, as the behavior will
differ. This type MUST be publicly exported from `crate::serde`.

r[jeb-value.serde.core.convert]
When the `serde` feature is enabled, `jeb_value::Value` MUST implement
`Self::from_serde(T: impl serde::Serialize) -> Result<Self, E>` and
`Self::to_serde(&self) -> T where T: serde::Deserialize`, which convert between
`jeb_value::Value` and any serde-serializable/deserializable type `T` using the
`ValueSerializer` and `ValueDeserializer` types respectively.

r[jeb-value.serde.core.bytes-representation]
When the `serde` feature is enabled, `jeb_value::Bytes`'s implementations of
`serde::Serialize` and `serde::Deserialize` MUST be compatible with `serde_bytes`
crate's representation for byte strings (i.e. it should support the
bytes-specific serde logic, not only the generic sequence logic).

r[jeb-value.serde.core.bytes-map-representation]
When the `serde` feature is enabled, `jeb_value::BytesMap`'s implementations of
`serde::Serialize` and `serde::Deserialize` MUST serialize and deserialize map
keys using `serde_bytes` crate's representation for byte strings (i.e. it should
support the bytes-specific serde logic, not only the generic sequence logic).

## Facet (`jeb-value.facet.`)

### Core (`jeb-value.facet.core.`)

r[jeb-value.facet.core.facet]
When the `facet` feature is enabled, `jeb_value::Value` and all variant types
MUST implement `Facet`.
