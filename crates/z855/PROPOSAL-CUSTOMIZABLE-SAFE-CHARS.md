# Proposal: Customizable Safe Character Sets

## Current State

All z855 encoders currently use a hard-coded list of 90 safe characters:
- The 85-character Z85 alphabet (always used for encoding)
- 5 additional characters used for escapes and passthrough: `,`, `;`, `|`, `~`, `_`

This set is optimal for general text interchange but may not suit all use cases.

## Proposed Change

Allow users to customize the set of safe characters (beyond the Z85 alphabet) for passthrough and escape sequences.

## Default Behavior

When the safe character set parameter is **unspecified or undefined**, encoders use the current hard-coded 90-character set. This preserves backward compatibility and provides sensible defaults for text interchange.

## Custom Safe Character Sets

### Specification Format

**TypeScript interface:**
```typescript
safeChars?: Iterable<number | string>
```

Where elements are:
- Integer numbers between 0-255 (byte values)
- Single-character strings with code points 0-127 (ASCII only)

Values outside these ranges throw an error immediately.

**Rust interface:**
```rust
safe_chars: impl IntoIterator<Item = u8>
```

Any iterable producing 8-bit unsigned integers.

### Validation Rules

**Immediate validation** (before encoding begins):
- ✅ Values must be in range (0-255 for bytes, 0-127 for char code points)
- ✅ Out-of-range values throw an error
- ✅ Duplicates are silently ignored
- ✅ Z85 alphabet characters are silently accepted (already implicit)

**No deferred validation**: Configuration errors fail immediately, not during encoding.

### Implicit Z85 Alphabet

The 85-character Z85 alphabet is **always available** regardless of the provided safe set. Users cannot disable it (it's the encoding alphabet, not a safe passthrough character). Whether users explicitly include Z85 characters in their safe set has no effect on behavior.

### Empty Safe Set

When the safe character set is **explicitly set to empty** (not omitted/undefined):
- Output will be pure Z85 encoding with no escapes
- No passthrough sequences possible
- No escape characters available
- Only standard 4-byte-aligned Z85 encoding used

This is distinct from the default (90 characters).

### Non-ASCII Safe Characters

Users may specify byte values >127 (non-ASCII) in the safe set. This means:
- Output may contain binary data (not valid text)
- User has implicitly opted into binary output
- Useful for cases like "allow everything except null byte"

## Additional Encoder Configuration

Beyond safe character customization, two additional encoder options enable composability and resource control.

### Concatenatable Mode

**Option name:** `concatenatable` (boolean, default `false`)

**Current behavior:** Encoded output may use end-of-stream optimization (raw data until end without length prefix) and may not be 4-byte aligned at stream end.

**Problem for composability:** 
1. End-of-stream indicator affects interpretation of any data concatenated afterward
2. Non-aligned boundaries break Z85 decoding when chunks are joined

**Proposed option:** When set to `true`, produce output that can be safely concatenated:

**Encoder behavior:**
1. Disable end-of-stream optimization (always use length-prefixed raw segments)
2. If final encoded block is not 4-byte aligned, pad the **beginning** with `#` characters

**Why `#` padding works:**
- `#` is a valid Z85 alphabet character
- Cannot naturally appear at the beginning of a Z85-encoded block (guaranteed by encoding math)
- Acts as unambiguous padding signal that decoder can strip

**Decoder requirement:**
- Decoder must strip leading `#` characters from **any block at any position**
- This enables concatenated chunks to have padding at internal boundaries
- Decoder change required regardless of encoder option (must support legacy concatenatable output)

**Benefit:** Encoded chunks can be concatenated freely without alignment constraints:
```typescript
const chunk1 = z855(data1, { concatenatable: true })  // 7 bytes → padded
const chunk2 = z855(data2, { concatenatable: true })  // 5 bytes → padded
const combined = chunk1 + chunk2  // Valid z855-encoded output
```

### Configurable Maximum Raw Segment Length

**Option name:** `maxRawSegmentLength` (integer, default `65536` = 64 KB)

**Current behavior:** Encoder limits raw passthrough segments to 64 KB by default. Larger raw sequences are broken into multiple segments or encoded as Z85.

**Motivation:** Different use cases optimize for different properties:
- Small limit → more frequent escape sequences, larger output for raw-heavy data
- Large limit → fewer escape sequences, smaller output for raw-heavy data
- Configurable limit allows tuning escape frequency vs. output size

**Proposed option:** Allow configuring this limit to any value between `0` and decoder maximum (`Number.MAX_SAFE_INTEGER` in JS, effectively unlimited).

**Valid range:**
- Minimum: `0` (no raw passthrough at all, pure Z85 encoding)
- Maximum: `9007199254740991` (JS max safe integer, decoder's enforced maximum)
- Values `0-3` are effectively equivalent (too small for any escape sequence)

**Special value handling:**
- No explicit "unlimited" needed; users pass `Number.MAX_SAFE_INTEGER` if desired
- Library may provide constant: `Z855_MAX_RAW_LENGTH = Number.MAX_SAFE_INTEGER`

**Example usage:**
```typescript
// No raw passthrough, pure Z85
z855(data, { maxRawSegmentLength: 0 })

// Frequent mode switching
z855(data, { maxRawSegmentLength: 256 })

// Default behavior
z855(data, { maxRawSegmentLength: 65536 })

// Minimize escape sequences
z855(data, { maxRawSegmentLength: Number.MAX_SAFE_INTEGER })
```

**Interaction with safe characters:** If safe set is empty, raw passthrough is impossible regardless of this limit. This option only affects behavior when escapes are available.

## Library Interface Changes

### Binary-Returning Functions (New)

Core encoding functions that always return binary data (`Uint8Array` / `Vec<u8>`):

```typescript
function z855Binary(
  input: Uint8Array,
  options?: {
    safeChars?: Iterable<number | string>
    disableEndOfStreamRaw?: boolean
    maxRawSegmentLength?: number
  }
): Uint8Array

function decodeBinary(encoded: Uint8Array): Uint8Array
```

```rust
struct Z855Options {
    safe_chars: Option<Vec<u8>>,
    disable_end_of_stream_raw: bool,
    max_raw_segment_length: usize,
}

fn z855_binary(input: &[u8], options: Z855Options) -> Vec<u8>
fn decode_binary(encoded: &[u8]) -> Vec<u8>
```

These functions accept any safe character configuration including non-ASCII values.

### Text-Returning Wrappers (Existing + Modified)

Convenience wrappers that return text strings:

```typescript
function z855(
  input: Uint8Array,
  options?: {
    safeChars?: Iterable<number | string>
    disableEndOfStreamRaw?: boolean
    maxRawSegmentLength?: number
  }
): string

function decode(encoded: string): Uint8Array
```

These wrappers:
1. **Validate before encoding**: Check if any safe character value is >127
2. **Throw immediately** if non-ASCII characters are specified
3. Otherwise, call the binary function and convert result to string

This ensures text-returning functions never produce invalid text output.

## Encoder Behavior

### Escape Character Selection

When multiple escape characters are available in the safe set, the encoder follows the **existing prioritization logic**. This proposal simply narrows the search space to available options.

Current escape priority (when available):
- `~` (7-byte passthrough)
- `_` (6-byte passthrough)
- `;` (5-byte passthrough)
- `,` (4-byte passthrough)
- `|` (length-prefixed passthrough)

If certain escape characters are not in the safe set, those escape lengths are not available.

### Determinism

Encoding behavior remains **fully deterministic**. Given the same input data and same safe character set, the encoder always produces identical output. The existing deterministic logic is unchanged; this proposal only narrows the options the encoder can choose from.

## Use Cases

### Pure Z85 (no escapes)
```typescript
z855(data, { safeChars: [] })
// Output: standard Z85 encoding only
```

### Allow all bytes except null
```typescript
z855Binary(data, { safeChars: [...Array(255).keys()].slice(1) })
// Output: binary data, may contain any byte except 0x00
```

### CSV-safe subset
```typescript
z855(data, { safeChars: '~_;' })
// Output: text, avoids , and | which have special meaning in CSV
```

### ASCII printable only
```typescript
z855(data, { safeChars: [...' ~'.charCodeAt(0)] })
// Output: text, all characters in range 0x20-0x7E
```

### Concatenable streaming chunks
```typescript
const chunks = largeData.match(/.{1,1024}/g)  // 1KB chunks
const encoded = chunks.map(chunk => 
  z855(chunk, { disableEndOfStreamRaw: true })
).join('')
// Each chunk is independently valid and concatenable
```

### Aggressive raw passthrough (minimize escaping)
```typescript
z855(data, { maxRawSegmentLength: Number.MAX_SAFE_INTEGER })
// Use largest possible raw segments, minimize escape overhead
```

### Frequent escaping for error detection
```typescript
z855(data, { maxRawSegmentLength: 256 })
// Small segments increase opportunities to detect corruption
```

### Pure Z85 via segment limit
```typescript
z855(data, { maxRawSegmentLength: 0 })
// Alternative to safeChars: [], disables all raw passthrough
```

## Implementation Scope

### Minimal Encoder (min.mjs)

The minimal encoder retains the **hard-coded 90-character default only**. No customization support. This keeps it small and focused.

### Full Encoders (TypeScript/Rust)

Full implementations provide the customizable interface described above. These are used via library imports and CLI tools where flexibility is valuable.

### CLI Interface

CLI customization is **explicitly out of scope** for this proposal. We will address command-line safe character specification in a separate design document.

## Migration Path

Existing code continues to work unchanged:
- Calls without `safeChars` parameter use default 90-character set
- Existing function signatures remain valid
- New binary functions are additions, not replacements
- Text wrappers gain optional parameter but maintain backward compatibility

## Open Questions

None. Specification is complete pending implementation.

## Summary

This proposal enables z855 to handle diverse encoding requirements through three orthogonal configuration dimensions:

1. **Safe character sets** - Control which bytes can pass through unencoded
2. **Stream composability** - Disable end-of-stream optimization for concatenable chunks
3. **Raw segment sizing** - Tune escape frequency vs. segment size

Design principles maintained:
- Preserving backward compatibility (defaults unchanged)
- Maintaining deterministic behavior
- Validating configuration eagerly (developer-friendly)
- Separating binary and text output concerns
- Keeping the minimal encoder simple

The customizable configuration transforms z855 from a fixed-configuration encoder into a flexible encoding framework suitable for diverse use cases: domain-specific safe character constraints, composable streaming output, and resource-aware segment sizing.
