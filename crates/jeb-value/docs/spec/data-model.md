# jeb-value Data Model Specification

This document specifies the data model for `jeb-value`, an extended JSON-like value system.

## Core Value Types

r[value.types.null status=stable level=must]
The value system MUST support a null/unit type representing the absence of a value.

r[value.types.bool status=stable level=must]
The value system MUST support boolean values (true and false).

r[value.types.number status=stable level=must]
The value system MUST support 64-bit floating-point numbers, excluding NaN and positive/negative Infinity but including negative zero.

r[value.types.bytes status=stable level=must]
The value system MUST support binary byte strings as a distinct type from text strings.

r[value.types.text status=stable level=must]
The value system MUST support UTF-8 text strings.

r[value.types.array status=stable level=must]
The value system MUST support arrays of arbitrary values maintaining insertion order.

r[value.types.bytes-map status=stable level=must]
The value system MUST support order-preserving maps with binary byte string keys and arbitrary values.

r[value.types.text-map status=stable level=must]
The value system MUST support order-preserving maps with text string keys and arbitrary values.

## Equality and Hashing

r[value.equality.structural status=stable level=must]
Values MUST support structural equality comparison where two values are equal if they have the same type and content.

r[value.equality.hashable status=stable level=must]
Values MUST be hashable such that equal values produce equal hash values.

r[value.hash.discriminant status=stable level=must]
The hash implementation MUST use a unique discriminant byte for each value type to ensure values of different types produce different hashes.

## Serde Integration

r[value.serde.serialize status=stable level=must]
Values MUST be serializable using serde's serialization framework.

r[value.serde.deserialize status=stable level=must]
Values MUST be deserializable from serde's deserialization framework.

r[value.serde.round-trip status=stable level=must]
Serializing a value and then deserializing it MUST produce an equivalent value (round-trip compatibility).

r[value.serde.json-compat status=stable level=should]
The value system SHOULD maintain compatibility with serde_json for standard JSON types (null, bool, number, string, array, object).

## Type Conversions

r[value.from.primitives status=stable level=must]
The value system MUST provide From implementations for common primitive types (bool, integers, strings) to enable ergonomic value construction.

r[value.try-unwrap status=stable level=must]
The value system MUST provide safe methods to attempt to extract specific types from values, returning errors for type mismatches.
