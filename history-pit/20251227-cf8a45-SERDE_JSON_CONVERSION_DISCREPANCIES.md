# serde_json Conversion Equivalence - RESOLVED ✅

## Summary

**RESOLVED**: The direct `From<serde_json::Value>` implementations and serde's `serialize`/`deserialize` traits now produce equivalent results!

The issue was caused by using derived `#[derive(Deserialize)]` with `#[serde(untagged)]` on the `Value` enum, which tried variants in order. Since `Bytes` came before `Array` and accepted sequences, arrays were incorrectly captured as bytes.

## Solution

Implemented a **manual `impl<'de> Deserialize<'de> for Value`** that maps serde's data model directly:
- `visit_seq` → `Value::Array` (always)
- `visit_bytes` → `Value::Bytes` (always)
- `visit_map` → `Value::TextMap` or `Value::BytesMap` (based on first key type)

## Test Results

**All 20 tests pass!** ✅

### What Works
- ✅ Null values
- ✅ Booleans
- ✅ Numbers (unsigned, signed, floats)
- ✅ Zero sign distinction (-0.0 vs +0.0)
- ✅ Strings
- ✅ **Empty arrays** (now correctly `Array([])` instead of `Bytes([])`)
- ✅ **Numeric arrays** (now correctly `Array([...])` instead of `Bytes([...])`)
- ✅ Objects (empty and nested)
- ✅ Complex nested structures
- ✅ Round-trip conversions

### Expected Normalization

JSON normalizes positive `Signed` integers to `Unsigned` since JSON doesn't distinguish signed/unsigned:
- `Signed(0)` → `Unsigned(0)`
- `Signed(i64::MAX)` → `Unsigned(9223372036854775807)`

This is expected JSON behavior and tests account for it.

## Files Changed

1. **crates/jeb-values/src/value.rs**
   - Removed `Deserialize` from derive (kept `Serialize` with `#[serde(untagged)]`)

2. **crates/jeb-values/src/serde/deserialize.rs**
   - Added manual `impl<'de> Deserialize<'de> for Value`
   - Created `ValueVisitor` with all visitor methods

3. **crates/jeb-values/src/serde/error.rs**
   - Renamed `Error` to `SerdeError` for clarity

4. **crates/jeb-values/tests/serde_json_from_equivalence.rs**
   - Added comprehensive test suite (20 tests)
   - Tests both direct `From` and serde deserialization
   - Handles expected JSON normalization

## Test Command

```bash
cargo test -p jeb-values --test serde_json_from_equivalence
```

All 80 tests in jeb-values pass! ✅
