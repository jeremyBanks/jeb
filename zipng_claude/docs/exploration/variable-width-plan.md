# Variable Width Implementation Plan

## Overview

Replace the fixed ROW_WIDTH=13 approach with dynamic width calculation that produces
approximately square images (with height >= width preference).

## Key Changes

### 1. Width Calculation

```rust
fn calculate_row_width(total_data_estimate: usize) -> usize {
    let ideal = 2.0 + (4.0 + total_data_estimate as f64).sqrt();
    width.floor().max(MIN_ROW_WIDTH)  // MIN_ROW_WIDTH = 40
}
```

### 2. Remove Fixed Constants

**Remove:**
- `const ROW_WIDTH: usize = 13`
- `const DATA_PER_BLOCK: usize = ROW_WIDTH - 4`
- `const FILTERED_ROW_SIZE: usize = ROW_WIDTH + 1`

**Replace with:**
- `MIN_ROW_WIDTH = 40` (minimum safe width)
- Calculate others dynamically based on chosen width

### 3. Modify `build_polyglot` Function

Current signature:
```rust
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    _width: u32,  // Currently ignored!
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8>
```

New approach:
1. First pass: estimate total data size from files
2. Calculate optimal row width
3. Second pass: build aligned data with that width
4. All helper functions receive `row_width` as parameter

### 4. Modify `build_aligned_data` Function

Current:
```rust
fn build_aligned_data(files: &[(&[u8], &[u8])]) -> (...)
```

New:
```rust
fn build_aligned_data(files: &[(&[u8], &[u8])], row_width: usize) -> (...)
```

Changes inside:
- Use `row_width` instead of `ROW_WIDTH` constant
- `data_per_block = row_width - 4`
- `filtered_row_size = row_width + 1`
- All alignment calculations use the parameter

### 5. Modify `encode_as_deflate_blocks` Function

Current:
```rust
fn encode_as_deflate_blocks(body: &[u8]) -> Vec<u8>
```

New:
```rust
fn encode_as_deflate_blocks(body: &[u8], row_width: usize) -> Vec<u8>
```

### 6. Modify `write_local_header` Function

Current approach uses tricks where filter bytes at positions 13 and 27 provide
mod_date_high and name_len_high.

**With variable width >= 40, filter bytes land AFTER the header, so:**
- Write complete standard ZIP local header (all fields fully specified)
- No filter-byte tricks needed
- Use extra field for padding to align content to row boundary

### 7. Modify `add_smart_filter_bytes` Function

Current:
```rust
fn add_smart_filter_bytes(data: &[u8], row_width: usize, final_rows: &HashSet<usize>) -> Vec<u8>
```

This already takes `row_width` as parameter - good!

### 8. Offset Calculation

The offset calculation in `build_polyglot` needs row_width:
```rust
let rows_before = orig_pos / row_width;
let filtered_pos = orig_pos + rows_before + 1;
```

## Data Flow

```
1. Estimate total size:
   - Sum file sizes
   - Add ~40 bytes header per file
   - Add alignment padding estimate

2. Calculate width:
   - width = floor(2 + sqrt(4 + total_estimate))
   - width = max(width, 40)

3. Build data:
   - For each file:
     - Align to row boundary
     - Write standard ZIP header
     - Add extra field padding to align content
     - Encode content as deflate blocks

4. Calculate final dimensions:
   - height = ceil(total_data / row_width)
   - Pad final row if needed

5. Build PNG with calculated dimensions
```

## Edge Cases

1. **Very small content (<1KB)**: width = 40 (minimum), produces tall narrow image
2. **Content near 42KB limit**: width ≈ 200, produces ~200×200 image
3. **Single tiny file**: May have significant padding overhead

## Testing Plan

1. Test with various content sizes: 100B, 1KB, 10KB, 30KB
2. Verify images are approximately square (height >= width)
3. Verify ZIP extraction works
4. Verify PNG is valid
5. Compare dimensions to expected calculations

## Rollback Plan

Keep the old implementation available (maybe as `build_polyglot_narrow`) in case
the new approach has issues.
