# Tracey Rule Review Examples

## Well-Scoped Rules (Good Examples)

These rules follow the single-responsibility principle - each rule specifies
one, clear requirement.

### Trait Implementation Rules (Individual)

```
r[jeb-value.value.clone]
`Value` MUST implement `Clone`.
```

```
r[jeb-value.value.debug]
`Value` MUST implement `Debug`.
```

```
r[jeb-value.value.must-use]
`Value` MUST be marked `#[must_use]`.
```

```
r[jeb-value.bytes.struct]
The `Bytes` variant type MUST be `#[repr(transparent)]` around a `Vec<u8>`.
```

### Type Conversion Rules (Focused)

```
r[jeb-value.variants.transparent]
Each variant type MUST be marked `#[repr(transparent)]`.
```

These are good models to follow - one requirement per rule.

---

## Overly Broad Rules (Candidates for Splitting)

### Issue 1: Multiple Traits in One Rule

**Rule**: `r[jeb-value.value.cmp]`

**Current Text**:

```
`Value` MUST implement `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and `Hash`, with
correct non-panicking behavior for all possible values.
```

**Problems**:

- Bundles 5 different trait implementations into one rule
- Harder to track which specific traits are implemented
- Tests for each trait might be in different places
- Partial implementation is unclear (which 3 of 5 traits are done?)

**Suggested Split**:

- r[jeb-value.value.eq]: `` `Value` MUST implement `Eq`. ``
- r[jeb-value.value.partial-eq]: `` `Value` MUST implement `PartialEq`. ``
- r[jeb-value.value.ord]: `` `Value` MUST implement `Ord`. ``
- r[jeb-value.value.partial-ord]: `` `Value` MUST implement `PartialOrd`. ``
- r[jeb-value.value.hash]:
  `` `Value` MUST implement `Hash` with correct non-panicking behavior. ``

**Benefits**:

- Each trait implementation can be tracked independently
- Tests for each trait have their own verification rule
- `tracey coverage` will show which specific traits are missing

---

### Issue 2: Multiple Type Conversions Bundled

**Rule**: `r[jeb-value.bytes.from-primitive]`

**Current Text**:

```
The `Bytes` variant type MUST implement `From<T>` for all of primitive types
`()`, `bool`, `f32`, `f64`, `char` and all integer types. The unit type MUST
map to an empty byte string, `false` to a single zero byte, `true` to a single
byte with value `0x01`, and all numeric types to their big-endian byte
representations.
```

**Problems**:

- Covers 13+ different type conversions (unit, bool, 6 floats/ints, plus all
  integer types)
- Mixes type conversions with their specific byte representations
- Hard to test - one failing conversion fails the whole rule
- Coverage metrics unclear - is it 0% if 1 type fails, or proportional?

**Suggested Split**:

- r[jeb-value.bytes.from-unit]:
  `` `Bytes` MUST implement `From<()>` mapping to empty byte string. ``
- r[jeb-value.bytes.from-bool]:
  `` `Bytes` MUST implement `From<bool>`, mapping false to 0x00 and true to 0x01. ``
- r[jeb-value.bytes.from-float]:
  `` `Bytes` MUST implement `From<f32>` and `From<f64>` using big-endian representation. ``
- r[jeb-value.bytes.from-char]:
  `` `Bytes` MUST implement `From<char>` using UTF-8 encoding. ``
- r[jeb-value.bytes.from-integers]:
  `` `Bytes` MUST implement `From<T>` for all signed and unsigned integer types using big-endian representation. ``

**Benefits**:

- Each type conversion is independently testable
- Can see which specific type conversions are missing
- More realistic coverage tracking

---

### Issue 3: Complex Conditional Logic

**Rule**: `r[jeb-value.variants.mut]`

**Current Text**:

```
If and only if a variant type implements infallible `From<INNER>` and implements
`Borrow<INNER>`, then it MUST also implement `BorrowMut<INNER>`, `AsMut<INNER>`,
and `DerefMut<Target=INNER>`, otherwise it MUST NOT implement any of those
traits.
```

**Problems**:

- Complex if/then/else with two sides
- "Infallible From<INNER>" requires understanding of related rules
- Combines both positive requirement (must implement) and negative requirement
  (must not)
- Hard to verify - what's the test for "MUST NOT implement"?

**Suggested Split**:

- r[jeb-value.variants.mut-precondition]:
  ``If a variant type implements infallible `From<INNER>` AND implements `Borrow<INNER>`, then the preconditions for mutable trait implementations are satisfied.``
- r[jeb-value.variants.borrow-mut]:
  ``If variant preconditions are satisfied, the variant MUST implement `BorrowMut<INNER>`.``
- r[jeb-value.variants.as-mut]:
  ``If variant preconditions are satisfied, the variant MUST implement `AsMut<INNER>`.``
- r[jeb-value.variants.deref-mut]:
  ``If variant preconditions are satisfied, the variant MUST implement `DerefMut<Target=INNER>`.``
- r[jeb-value.variants.no-mut-without-preconditions]:
  ``If a variant type does NOT satisfy the preconditions, it MUST NOT implement `BorrowMut`, `AsMut`, or `DerefMut`.``

**Benefits**:

- Each trait implementation can be verified separately
- Precondition is explicit and reusable
- Negative requirement is its own testable rule

---

### Issue 4: Multiple Scenarios/Branches

**Rule**: `r[jeb-value.variant.try-from-other-via-inner]`

**Current Text**:

```
Given two variant types `V` and `W`, and their respective inner types `INNER_V`
and `INNER_W`, then if `V` implements `From<INNER_W>` then `V` MUST also
implement `From<W>` by unwrapping `W` to get its inner type and then using
the `From<INNER_W>` implementation to convert it to `V`. Otherwise, if `V`
implements `TryFrom<INNER_W>`, then `V` MUST also implement `TryFrom<W>` by
unwrapping `W` to get its inner type and then using the `TryFrom<INNER_W>`
implementation to convert it to `V`.
```

**Problems**:

- Two different scenarios (From vs TryFrom)
- Each scenario has its own implementation pattern
- Testing needs to cover both paths
- Implements two different trait pairs in one rule

**Suggested Split**:

- r[jeb-value.variant.from-other-via-inner]:
  ``Given variant types `V` and `W` with inner types `INNER_V` and `INNER_W`: If `V` implements `From<INNER_W>`, then `V` MUST also implement `From<W>` by unwrapping `W` and using the `From<INNER_W>` implementation.``
- r[jeb-value.variant.try-from-other-via-inner]:
  ``If `V` implements `TryFrom<INNER_W>`, then `V` MUST also implement `TryFrom<W>` by unwrapping `W` and using the `TryFrom<INNER_W>` implementation.``

**Benefits**:

- Each scenario is independently specified and testable
- Implementation code can reference the specific rule it satisfies
- Test coverage per scenario is clearer

---

### Issue 5: Complex Type System Rules

**Rule**: `r[jeb-value.number.cmp]`

**Current Text**:

```
The `Number` variant type's implementations of `Eq`, `PartialEq`, `Ord`,
`PartialOrd`, and `Hash` MUST NOT delegate to the inner `f64` type, but should
instead delegate comparison and equality to `f64::total_cmp` and should delegate
the `Hash` implementation to the result of `.to_be_bytes()`. This ensures a
total ordering and distinct hashing for all possible `f64` values.
```

**Problems**:

- Specifies implementations for 5 different traits
- Includes specific implementation details for each
- Mix of "MUST NOT" and "should instead" language
- Implementation guidance mixed with requirements

**Suggested Split**:

- r[jeb-value.number.not-delegate-f64]:
  `` `Number` comparison and hashing MUST NOT directly delegate to the inner `f64` type. ``
- r[jeb-value.number.eq-total-cmp]:
  `` `Number` MUST implement `Eq` and `PartialEq` using `f64::total_cmp` to ensure consistent behavior for NaN. ``
- r[jeb-value.number.ord-total-cmp]:
  `` `Number` MUST implement `Ord` and `PartialOrd` using `f64::total_cmp` for a total ordering. ``
- r[jeb-value.number.hash-be-bytes]:
  `` `Number` MUST implement `Hash` by delegating to the hash of `.to_be_bytes()` to ensure distinct hashing for all f64 values. ``

**Benefits**:

- Each trait is its own verifiable requirement
- Specific implementation guidance is clearer
- Can verify the "MUST NOT" requirement separately
- Each test focuses on one aspect

---

## Pattern Recognition Quick Reference

When reviewing rules, look for these red flags:

| Pattern                  | Example                                        | Action                            |
| ------------------------ | ---------------------------------------------- | --------------------------------- |
| Multiple traits listed   | "Eq, PartialEq, Ord, PartialOrd, Hash"         | Split by trait                    |
| Type categories          | "for all primitive types", "all integer types" | Split by type                     |
| "and" separators         | "implement X AND implement Y AND implement Z"  | Split into separate rules         |
| If/then/else             | "If X then must Y, otherwise must Z"           | Split by condition                |
| Multiple specific values | "false → 0x00, true → 0x01, unit → empty"      | Consider splitting by value/type  |
| Complex conditions       | "if and only if ... and ... then ... and ..."  | Break into precondition + effects |

---

## Rule ID Naming Convention

This project uses hierarchical rule IDs:

```
r[namespace.entity.aspect]

- namespace: jeb-value, jeb-time, etc.
- entity: the thing being specified (value, bytes, number, variants, etc.)
- aspect: the specific requirement (clone, debug, cmp, from-primitive, etc.)
```

When splitting rules, maintain this hierarchy:

```
OLD: r[jeb-value.value.cmp]

NEW:
- r[jeb-value.value.eq]
- r[jeb-value.value.partial-eq]
- r[jeb-value.value.ord]
```

If a rule needs sub-aspects, extend with hyphens:

```
r[jeb-value.bytes.from-primitive]  →  r[jeb-value.bytes.from-bool]
                                   →  r[jeb-value.bytes.from-integers]
```
