# serde_json Conversion Discrepancies

## Summary

**IMPORTANT**: The direct `From<serde_json::Value>` implementations in `crates/jeb-values/src/serde_json/mod.rs` are **NOT equivalent** to using serde's `serialize`/`deserialize` traits.

I ran comprehensive tests comparing both approaches and found **7 test failures** showing different behavior.

## Test Results

- ✅ **13 tests passed** (primitives, nested objects, complex structures)
- ❌ **7 tests failed** (arrays and round-trips)

## Key Discrepancy: Array vs Bytes

The most critical difference is how **empty and numeric arrays** are handled:

### Empty Array Example
```rust
let json = serde_json::json!([]);

// Direct From<serde_json::Value>
let direct: Value = json.clone().into();
// Result: Value::Array([])

// Via serde deserialize
let via_serde: Value = serde_json::from_value(json).unwrap();
// Result: Value::Bytes(Bytes([]))  ❌ DIFFERENT!
```

### Numeric Array Example
```rust
let json = serde_json::json!([[1, 2], [3, 4]]);

// Direct From<serde_json::Value>
// Result: Value::Array([
//     Value::Array([Unsigned(1), Unsigned(2)]),
//     Value::Array([Unsigned(3), Unsigned(4)])
// ])

// Via serde deserialize
// Result: Value::Array([
//     Value::Bytes(Bytes([1, 2])),  ❌ Arrays of u8 become Bytes!
//     Value::Bytes(Bytes([3, 4]))
// ])
```

## Root Cause

The serde implementation has special logic that converts arrays of u8 into `Value::Bytes`:
- When deserializing via serde, arrays containing only u8-sized integers are interpreted as byte arrays
- The direct `From` implementation treats them as regular arrays

## Affected Tests

1. `test_from_json_array_empty` - Empty arrays
2. `test_from_json_array_nested` - Nested numeric arrays
3. `test_from_json_mixed_array_types` - Mixed content with numeric arrays
4. `test_from_json_object_empty` - Probably related to nested empty arrays
5. `test_roundtrip_primitives` - Round-trip conversions
6. `test_roundtrip_arrays` - Array round-trips
7. `test_roundtrip_objects` - Object round-trips (likely containing arrays)

## What Works Correctly

The following conversions **are equivalent** between both approaches:
- ✅ Null values
- ✅ Booleans
- ✅ Numbers (unsigned, signed, floats)
- ✅ Strings
- ✅ Objects with non-array values
- ✅ Deeply nested structures (without numeric arrays)
- ✅ Zero sign distinction (-0.0 vs +0.0)

## Recommendation

The comment in `serde_json/mod.rs` claiming these are equivalent is **incorrect**. You need to decide:

1. **Keep both implementations** - Document that they behave differently for arrays
2. **Make them equivalent** - Either:
   - Update the direct `From` to match serde's byte array logic, OR
   - Change serde's Deserialize impl to not convert arrays to bytes
3. **Remove the direct `From`** - Just use serde for consistency

## Test File

The comprehensive test suite is available at:
`crates/jeb-values/tests/serde_json_from_equivalence.rs`

Run with:
```bash
cargo test -p jeb-values --test serde_json_from_equivalence
```
