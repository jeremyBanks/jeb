# `jeb-value` specification

This is the [Tracey](https://crates.io/crates/tracey) specification for the
`jeb-value` crate. Run the `tracey` command-line tool to evaluate coverage.

## Cargo Features

r[jeb-value.dependencies.optional]  
Any optional dependency MUST have a correspondingly-named Cargo feature to 
enable it. If an optional dependency is known to depend on another optional 
dependency, their corresponding features MUST also have the same dependency 
relationship (e.g. if they were both dependencies of a crate, the
 `serde-json` feature would need to depend on `serde`).

Multiple optional dependencies MAY be grouped behind a single feature if
they're both required for a single set of functionality.

r[jeb-value.dependencies.limit-internal]  
This crate MUST NOT have any non-test dependencies on other crates within this 
workspace except for `jeb-common` (which MAY be added).

r[jeb-value.dependencies.cfg]  
Any use of an optional dependency MUST be gated behind the corresponding Cargo
feature using `cfg!`, `#[cfg ...]`, `#[cfg_attr ...]` or similar.

## `Value` (`jeb-value.value.`)

r[jeb-value.value.pub]  
The crate MUST publicly export the `Value` enum from its root (and nowhere else),
such that users might `use json_value::Value`.

r[jeb-value.value.src]  
`Value` must be defined in `src/value/mod.rs`. That file should not directly
contain anything except the definition of `Value` and any item macros (such as
derive) that we apply to it, and declarations or sub-modules. All related code,
even including inherent impls and trait implementations, must go in sub-modules
under `src/value/`.

r[jeb-value.value.enum-variants]  
`Value` MUST be an enum with variants `Null`, `Boolean`, `Number`, `Bytes`,
`String`, `Array`, `BytesMap`, and `StringMap`. This list is exhaustive.

r[jeb-value.value.variant-types]  
Each `Value` enum variant MUST be a single-element tuple variant over a type
(which we'll refer to as a "variant type") of the same name.

r[jeb-value.value.round-trip]  
If `Value` defines `From<T>` or `TryFrom<T>` for any type `T`, then `T` MUST
also implement `TryFrom<Value>` which can losslessly recover any values that
were converted using `From<T>` or a successful `TryFrom<T>`. This is a universal
requirement applying to all such conversions, regardless of whether the other
type is internal, external, or built-in.

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

r[jeb-value.variant.src]  
Each variant type must be defined in `src/VARIANT/mod.rs`, where variant is
the appropriate snake_case name. That file should not directly contain anything
except the definition of the variant type and any item macros (such as derive)
that we apply to it, and declarations or sub-modules. All related code, even
including inherent impls and trait implementations, must go in sub-modules under
`src/value/`.

r[jeb-value.variants.tuple]  
Each variant type MUST be a single-item tuple wrapping an inner value.

r[jeb-value.variant.value-from]  
`Value` MUST implement `From<T>` for each variant type, wrapping it in the
appropriate enum variant.

r[jeb-value.variant.value-as]  
`Value` MUST implement an `as_VARIANT(&self) -> Option<&VARIANT>` method for
each variant type.

r[jeb-value.variant.value-to]  
`Value` MUST implement a `to_VARIANT(&self) -> Option<VARIANT>` method for each
variant type.

r[jeb-value.variant.value-into]  
`Value` MUST implement an `into_VARIANT(self) -> Option<VARIANT>` method for
each variant type.

r[jeb-value.variant.value-unwrap]  
`Value` MUST implement an `unwrap_VARIANT(self) -> VARIANT` method for each
variant type, which panics if the `Value` is not of the expected variant type.

r[jeb-value.variant.round-trip]  
If a variant type `V` defines `From<T>` or `TryFrom<T>` for any type `T`, then
`T` MUST also implement `TryFrom<V>` which can losslessly recover any values
that were converted using `From<T>` or a successful `TryFrom<T>`. This is a
universal requirement applying to all such conversions, regardless of whether
the other type is internal, external, or built-in.

r[jeb-value.variant.try-from-inner]  
Each variant MUST implement `TryFrom<INNER>` for their wrapped inner type. This
may be implicit from a `From<INNER>` implementation or explicit if it's
fallible.

r[jeb-value.variant.try-from-other-via-inner]  
Given two variant types `V` and `W`, and their respective inner types `INNER_V`
and `INNER_W`, then if `V` implements `From<INNER_W>` then `V` MUST also
implement `From<W>` by unwrapping `W` to get its inner type and then using
the `From<INNER_W>` implementation to convert it to `V`. Otherwise, if `V`
implements `TryFrom<INNER_W>`, then `V` MUST also implement `TryFrom<W>` by
unwrapping `W` to get its inner type and then using the `TryFrom<INNER_W>`
implementation to convert it to `V`.

r[jeb-value.variants.constructor]  
If a variant implements infallible `From<INNER>`, it MUST also provide a public
`new(inner: INNER) -> Self` constructor function.

r[jeb-value.variants.as-ref]  
Each variant type MUST implement `AsRef<INNER>`.

r[jeb-value.variants.deref]  
Each variant type MUST implement `Deref<Target=INNER>`.

r[jeb-value.variants.inner-from]  
For each variant type, their inner type MUST implement `From<VARIANT>`.

r[jeb-value.variants.into-inner]  
Each variant type MUST implement `into_inner(self): INNER`.

r[jeb-value.variants.to-inner]  
Each variant type MUST implement `to_inner(&self): INNER`.

r[jeb-value.variants.as-inner]  
Each variant type MUST implement `as_inner(&self): &INNER`.

r[jeb-value.variants.into-named-inner]  
Each variant type MUST implement `into_INNER(self): INNER`, where INNER is the
appropriately-formatted inner type name (ignoring any generic parameters).

r[jeb-value.variants.to-named-inner]  
Each variant type MUST implement `to_INNER(&self): INNER`, where INNER is the
appropriately-formatted inner type name (ignoring any generic parameters).

r[jeb-value.variants.as-named-inner]  
Each variant type MUST implement `as_INNER(&self): &INNER`, where INNER is the
appropriately-formatted inner type name (ignoring any generic parameters).

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

r[jeb-value.variant.must-use]  
Each variant type MUST be marked `#[must_use]` unless specified otherwise for
that variant type.

### `Null`

r[jeb-value.null]  
The `Null` variant type MUST be a single-item tuple struct wrapping an inner
primitive unit value `()`.

r[jeb-value.null.must-use]  
The `Null` variant type MUST NOT be marked `#[must_use]`.

r[jeb-value.null.from-inner]  
The `Null` variant type MUST implement `From<()>`.

r[jeb-value.null.try-from-primitive]  
The `Null` variant type MUST implement `TryFrom<T>` where `T` is any of
`bool` (with only `false` being accepted), `f32` and `f64` (with only `+0.0`
being accepted), and all integer types (with only `0` being accepted).

r[jeb-value.null.try-from-vec]  
The `Null` variant type MUST implement `Vec<T>` (where T is _unconstrained_)
with only the empty vector being accepted.

r[jeb-value.null.try-from-indexmap]  
The `Null` variant type MUST implement `IndexMap<K, V>` (where K and V are
_unconstrained_) with only the empty map being accepted.

r[jeb-value.null.primitive-from]  
All of the primitive types `()`, `bool`, `f32`, `f64`, and all integer types,
and `Vec<T>` and `IndexMap<K, V>` (where T, K, and V are unconstrained)
MUST implement `From<Null>`, mapping to their respective default values.

### `Boolean`

r[jeb-value.boolean]  
The `Boolean` variant type MUST be a single-item tuple struct wrapping an
inner primitive `bool`.

r[jeb-value.boolean.from-inner]  
The `Boolean` variant type MUST implement `From<bool>`.

r[jeb-value.boolean.from-false]  
The `Boolean` variant type MUST implement `From<()>` (mapping to `false`).

r[jeb-value.boolean.try-from-primitive]  
The `Boolean` variant type MUST implement `TryFrom<T>` where `T` is any of
`f32` and `f64`, and all integer types, with (positive) zero being false,
positive one being true, and all other values being rejected.

### `Number`

r[jeb-value.number]  
The `Number` variant type MUST be a single-item tuple struct wrapping an
inner primitive `f64`.

r[jeb-value.number.finite]  
The `Number` variant type MUST only be constructible with finite `f64` values
(excluding NaN or Infinity, including `-0`). By enforcing this everywhere that a
value can be constructed, all operations on `Number` can safely assume the inner
value is always finite without needing to re-validate it.

r[jeb-value.number.constructor]  
The `Number` variant type MUST provide a public
`new(value: f64) -> Option<Self>` constructor function which returns `Some` if
the provided `f64` is finite, and `None` otherwise.

r[jeb-value.number.try-from-inner]  
The `Number` variant type MUST implement `TryFrom<f64>`, returning an error
if the provided `f64` is not finite.

r[jeb-value.number.no-from-inner]  
The `Number` variant type MUST NOT implement `From<f64>`.

r[jeb-value.number.cmp]  
The `Number` variant type's implementations of `Eq`, `PartialEq`, `Ord`,
`PartialOrd`, and `Hash` MUST NOT delegate to the inner `f64` type, but should
instead delegate comparison and equality to `f64::total_cmp` and should delegate
the `Hash` implementation to the result of `.to_be_bytes()`. This ensures a
total ordering and distinct hashing for all possible `f64` values.

### `Bytes`

r[jeb-value.bytes.struct]  
The `Bytes` variant type MUST be a single-item tuple struct wrapping an
inner `Vec<u8>`.

r[jeb-value.bytes.from-inner]  
The `Bytes` variant type MUST implement `From<Vec<u8>>`.

r[jeb-value.bytes.from-slice]
The `Bytes` variant type MUST implement `From<&[u8]>`.

r[jeb-value.bytes.from-iterator]
The `Bytes` variant type MUST implement `FromIterator<u8>>`.

r[jeb-value.bytes.from-slice-iterator]
The `Bytes` variant type MUST implement `FromIterator<&[u8]>>`.

r[jeb-value.bytes.from-primitive]
The `Bytes` variant type MUST implement `From<T>` for all of primitive types
`()`, `bool`, `f32`, `f64`, `char` and all integer types. The unit type MUST
map to an empty byte string, `false` to a single zero byte, `true` to a single
byte with value `0x01`, and all numeric types to their big-endian byte
representations.

### `String`

r[jeb-value.string.struct]  
The `String` variant type MUST be a single-item tuple struct wrapping an
inner `String`.

r[jeb-value.string.from-inner]  
The `String` variant type MUST implement `From<String>`.

r[jeb-value.string.from-str]  
The `String` variant type MUST implement `From<&str>`.

r[jeb-value.string.from-char-iterator]
The `String` variant type MUST implement `FromIterator<char>>`.

r[jeb-value.string.from-string-iterator]
The `String` variant type MUST implement `FromIterator<String>>`.

r[jeb-value.string.from-str-iterator]
The `String` variant type MUST implement `FromIterator<&str>>`.

### `Array`

r[jeb-value.array.struct]  
The `Array` variant type MUST be a single-item tuple struct wrapping an
inner `Vec<Value>`.

r[jeb-value.array.len]  
The `Array` variant type MUST implement `.len(&self) -> usize`.

r[jeb-value.array.is-empty]  
The `Array` variant type MUST implement `.is_empty(&self) -> bool`.

r[jeb-value.array.iter]
The `Array` variant type MUST implement `.iter(&self) -> impl Iterator<Item=&Value>`.

r[jeb-value.array.into-iterator]
The `Array` variant type MUST implement `IntoIterator<Item=Value>`.

r[jeb-value.array.from-inner]  
The `Array` variant type MUST implement `From<Vec<Value>>`.

r[jeb-value.array.from-slice]
The `Array` variant type MUST implement `From<&[Value]>`.

r[jeb-value.array.from-iterator]
The `Array` variant type MUST implement `FromIterator<Value>`.

### `BytesMap`

r[jeb-value.bytes-map.struct]  
The `BytesMap` variant type MUST be a single-item tuple struct wrapping an
inner `indexmap::IndexMap<Vec<u8>, Value>`.

r[jeb-value.bytes-map.len]  
The `BytesMap` variant type MUST implement `.len(&self) -> usize`.

r[jeb-value.bytes-map.is-empty]  
The `BytesMap` variant type MUST implement `.is_empty(&self) -> bool`.

r[jeb-value.bytes-map.index]  
The `BytesMap` variant type MUST implement `Index` delegating to the inner map.
XXX: However, it must also implement `Index` accepting a `Value`. Do these
requirements contradict?

r[jeb-value.bytes-map.get]
The `BytesMap` variant type MUST implement `.get` delegating to the inner map.
XXX: However, it must also implement `.get` accepting a `Value`. Do these
requirements contradict?

r[jeb-value.bytes-map.contains-key]
The `BytesMap` variant type MUST implement `.contains_key` delegating to the inner map.
XXX: However, it must also implement `.contains_key` accepting a `Value`. Do
these requirements contradict?

r[jeb-value.bytes-map.from-inner]  
The `BytesMap` variant type MUST implement
`From<indexmap::IndexMap<Vec<u8>, Value>>`.

r[jeb-value.bytes-map.keys]  
The `BytesMap` variant type MUST implement
`.keys(&self) -> impl Iterator<Item=&Vec<u8>>`.

r[jeb-value.bytes-map.values]  
The `BytesMap` variant type MUST implement
`.values(&self) -> impl Iterator<Item=&Value>`.

r[jeb-value.bytes-map.iter]  
The `BytesMap` variant type MUST implement
`.iter(&self) -> impl Iterator<Item=(&Vec<u8>, &Value)>`.

r[jeb-value.bytes-map.into-iterator]  
The `BytesMap` variant type MUST implement
`IntoIterator<Item=(Vec<u8>, Value)>`.

### `StringMap`

r[jeb-value.string-map.struct]  
The `StringMap` variant type MUST be a single-item tuple struct wrapping an
inner `indexmap::IndexMap<String, Value>`.

r[jeb-value.string-map.len]  
The `StringMap` variant type MUST implement `.len` delegating to the inner map.

r[jeb-value.string-map.is-empty]  
The `StringMap` variant type MUST implement `.is_empty` delegating to the inner map.

r[jeb-value.string-map.index]  
The `StringMap` variant type MUST implement `Index` delegating to the inner map.
XXX: However, it must also implement `Index` accepting a `Value`. Do these
requirements contradict?

r[jeb-value.string-map.get]
The `StringMap` variant type MUST implement `.get` delegating to the inner map.
XXX: However, it must also implement `.get` accepting a `Value`. Do these
requirements contradict?

r[jeb-value.string-map.contains-key]
The `StringMap` variant type MUST implement `.contains_key` delegating to the inner map.
XXX: However, it must also implement `.contains_key` accepting a `Value`. Do
these requirements contradict?

r[jeb-value.string-map.from-inner]  
The `StringMap` variant type MUST implement
`From<indexmap::IndexMap<String, Value>>`.

r[jeb-value.string-map.keys]  
The `StringMap` variant type MUST implement
`.keys(&self) -> impl Iterator<Item=&String>`.

r[jeb-value.string-map.values]  
The `StringMap` variant type MUST implement
`.values(&self) -> impl Iterator<Item=&Value>`.

r[jeb-value.string-map.iter]  
The `StringMap` variant type MUST implement
`.iter(&self) -> impl Iterator<Item=(&String, &Value)>`.

r[jeb-value.string-map.into-iterator]  
The `StringMap` variant type MUST implement
`IntoIterator<Item=(String, Value)>`.

## Serde

r[jeb-value.serde.optional]  
Any dependencies on `serde` and other `serde-*` ecosystem crates MUST be
optional, gated behind a `serde` Cargo feature.

r[jeb-value.serde.serialize]  
`Value` MUST implement `serde::Serialize`.

r[jeb-value.serde.deserialize]  
`Value` MUST implement `serde::Deserialize`.

r[jeb-value.serde.representation]  
`Value`'s implementations of `serde::Serialize` and `serde::Deserialize` must
be compatible with the (default) externally-tagged enum representation.

r[jeb-value.serde.serializer]
The crate MUST provide a `ValueSerializer` type implementing
`serde::ser::Serializer` which serializes an arbitrary serde-serializable type
into a `Value`. This MUST NOT use on our own `Value`'s `serde::Serialize`
implementation, as the behavior will differ. This type MUST be publicly exported
from `crate::serde`.

r[jeb-value.serde.deserializer]
The crate MUST provide a `ValueDeserializer` type implementing
`serde::de::Deserializer` which deserializes an arbitrary serde-deserializable
type from a `Value`. This MUST NOT use on our own `Value`'s `serde::Deserialize`
implementation, as the behavior will differ. This type MUST be publicly exported
from `crate::serde`.

r[jeb-value.serde.convert]  
`Value` MUST implement
`Self::from_serde(T: impl serde::Serialize) -> Result<Self, E>`
and `Self::to_serde(&self) -> T where T: serde::Deserialize`, which convert
between `Value` and any serde-serializable/deserializable type `T` using the
`ValueSerializer` and `ValueDeserializer` types respectively.

r[jeb-value.serde.bytes.representation]  
`Bytes`'s implementations of `serde::Serialize` and `serde::Deserialize` MUST
be compatible with `serde_bytes` crate's representation for byte strings (i.e.
it should support the bytes-specific serde logic, not only the generic sequence
logic).

r[jeb-value.serde.bytes-map.representation]  
`BytesMap`'s implementations of `serde::Serialize` and `serde::Deserialize` MUST
serialize and deserialize map keys using `serde_bytes` crate's representation
for byte strings (i.e. it should support the bytes-specific serde logic, not
only the generic sequence logic).

## Facet

r[jeb-value.facet.optional]  
Any dependencies on `facet` and other `facet-*` ecosystem crates MUST be
optional, gated behind a `facet` Cargo feature.

r[jeb-value.facet.facet]  
When the `facet` Cargo feature is enabled, `Value` and all variant types MUST
implement `Facet`.
