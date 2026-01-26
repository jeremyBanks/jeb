# Variable Width Implementation Plan v2

*Updated based on review feedback*

## Implementation Order

The code is currently in a broken transitional state. Here's the correct order:

### Phase 1: Add row_width parameter to all functions (keep constants as fallback)

1. Add `row_width: usize` parameter to:
   - `build_aligned_data(files, row_width)`
   - `encode_as_deflate_blocks(body, row_width)`
   - `write_local_header(...)` - complete rewrite needed

2. Calculate derived values inside functions:
   ```rust
   let data_per_block = row_width - 4;
   let filtered_row_size = row_width + 1;
   ```

### Phase 2: Add estimation and width calculation to build_polyglot

```rust
pub fn build_polyglot(...) -> Vec<u8> {
    // Step 0: Estimate total size and calculate optimal width
    let total_estimate = estimate_total_size(files);
    let row_width = calculate_row_width(total_estimate);

    // Step 1: Build pixel data with calculated width
    let (pixel_data, entry_infos, final_block_rows) = build_aligned_data(files, row_width);
    // ... rest unchanged but uses row_width
}
```

### Phase 3: Remove old constants

Remove `ROW_WIDTH`, `DATA_PER_BLOCK`, `FILTERED_ROW_SIZE` constants entirely.

## Size Estimation Formula

```rust
fn estimate_total_size(files: &[(&[u8], &[u8])]) -> usize {
    let mut total = 0;
    for (name, body) in files {
        // ZIP header: 30 bytes (fixed) + name length
        total += 30 + name.len();
        // Extra field padding: up to row_width bytes (estimate 40)
        total += 40;
        // File content: body + deflate overhead (4 bytes per ~36 bytes)
        total += body.len();
        total += (body.len() / 36 + 1) * 4;
    }
    total
}
```

## write_local_header Complete Rewrite

**Old approach (width=13):** Used filter bytes at positions 13 and 27 to provide certain header bytes.

**New approach (width>=40):** Write complete standard ZIP local header.

```rust
fn write_local_header(
    data: &mut Vec<u8>,
    name: &[u8],
    uncompressed_size: usize,
    compressed_size: usize,
    crc: u32,
    extra_len: usize,
) {
    // Standard 30-byte ZIP local file header
    data.extend_from_slice(b"PK\x03\x04");           // 0-3: signature
    data.extend_from_slice(&20_u16.to_le_bytes());   // 4-5: version needed
    data.extend_from_slice(&0_u16.to_le_bytes());    // 6-7: flags
    data.extend_from_slice(&8_u16.to_le_bytes());    // 8-9: compression (deflate)
    data.extend_from_slice(&0_u16.to_le_bytes());    // 10-11: mod time
    data.extend_from_slice(&0_u16.to_le_bytes());    // 12-13: mod date (FULL 2 bytes now!)
    data.extend_from_slice(&crc.to_le_bytes());      // 14-17: CRC-32
    data.extend_from_slice(&(compressed_size as u32).to_le_bytes()); // 18-21
    data.extend_from_slice(&(uncompressed_size as u32).to_le_bytes()); // 22-25
    data.extend_from_slice(&(name.len() as u16).to_le_bytes());  // 26-27: name len (FULL)
    data.extend_from_slice(&(extra_len as u16).to_le_bytes());   // 28-29: extra len
    data.extend_from_slice(name);                    // 30+: filename

    // Extra field for alignment padding
    if extra_len > 0 {
        // Use dummy extra field: ID=0x0000, size=extra_len-4, data=zeros
        if extra_len >= 4 {
            data.extend_from_slice(&0x0000_u16.to_le_bytes());
            data.extend_from_slice(&((extra_len - 4) as u16).to_le_bytes());
            data.resize(data.len() + extra_len - 4, 0);
        } else {
            data.resize(data.len() + extra_len, 0);
        }
    }
}
```

Note: This writes 30 bytes (not 28), plus name, plus extra field.

## Edge Cases

### Empty files array
```rust
if files.is_empty() {
    // Return minimal valid PNG+ZIP with no entries
    // Or return error
}
```

### Content exceeds 42KB limit
```rust
let total_content: usize = files.iter().map(|(_, b)| b.len()).sum();
if total_content > MAX_CONTENT_SIZE {
    // Option A: Return error
    // Option B: Log warning, proceed anyway (extraction may fail)
}
```

### Very small content
With MIN_ROW_WIDTH=40, very small content produces narrow images. This is acceptable.

## Offset Calculation

The offset calculation needs row_width:
```rust
// Convert original data position to filtered position
let rows_before = orig_pos / row_width;
let offset_in_row = orig_pos % row_width;
let filtered_pos = rows_before * (row_width + 1) + 1 + offset_in_row;

// Account for IDAT block headers
let idat_blocks_before = filtered_pos / 65535;
let file_offset = data_offset + filtered_pos + (idat_blocks_before * 5);
```

## Testing Matrix

| Content Size | Expected Width | Expected Height | Test |
|--------------|----------------|-----------------|------|
| 0 bytes      | 40             | 1+ (min)        | Edge |
| 100 bytes    | 40             | ~5              | Small |
| 1 KB         | 40             | ~30             | Small |
| 5 KB         | ~73            | ~73             | Medium |
| 10 KB        | ~102           | ~102            | Medium |
| 30 KB        | ~175           | ~175            | Large |
| 42 KB        | ~207           | ~207            | Limit |

Test with:
- `unzip -t` (verify integrity)
- `unzip -l` (list contents)
- `unzip` (extract)
- `file` command (verify PNG)
- Image viewer (verify renders)

## The `_width` Parameter

Current: `build_polyglot(files, _width, bit_depth, color_mode, palette)`

Options:
1. **Remove it** - API breaking change
2. **Ignore it** - Current behavior, confusing
3. **Use as minimum** - `width.max(MIN_ROW_WIDTH).max(user_width)`
4. **Use as override** - If non-zero, use exactly that width

Recommendation: Option 3 (use as minimum) for backward compatibility.
