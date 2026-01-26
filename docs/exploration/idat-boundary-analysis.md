# IDAT Boundary Analysis

## The Problem

IDAT uses stored deflate blocks with max 65535 bytes. The byte stream looks like:

```
[zlib header 2B] [deflate header 5B] [filtered data 65535B] [deflate header 5B] [filtered data ...] [adler32 4B]
```

The 5-byte deflate headers appear at fixed positions:
- Offset 2-6: first header (LEN=65535 or less)
- Offset 65542-65546: second header
- Offset 131082-131086: third header
- etc.

Between headers, there are 65535 bytes of filtered data (our rows).

ZIP readers parse the file from local headers. If a file's deflate content spans a header position,
those 5 bytes appear in the stream and corrupt it.

## Current Approach

Limit total content to ~42KB so there's only one deflate block. Simple but restrictive.

## Potential Solutions

### Solution A: Pad to Avoid Boundaries

For each file, check if its content would cross a 65535-byte boundary. If so, add padding
to push the file past the boundary.

**Pros**: Simple concept, unlimited total size
**Cons**: Some padding overhead, need to track positions carefully

**Implementation**:
1. Track current position in filtered data
2. Before each file, check if `current_pos + file_size` crosses a boundary
3. If so, add padding to reach the boundary
4. Boundary positions in filtered data: 65535, 131070, 196605, ...

### Solution B: Align Rows to Divide Boundaries

Choose row width such that `filtered_row_size` divides 65535.

65535 = 3 × 5 × 17 × 257

Possible filtered_row_sizes: 3, 5, 15, 17, 51, 85, 255, 257...
Corresponding row_widths: 2, 4, 14, 16, 50, 84, 254, 256...

With MIN_ROW_WIDTH=40, nearest options: 50 (51 divides 65535) or 84 (85 divides 65535).

**Problem**: Even with aligned rows, the 5-byte header still appears between rows.
ZIP would see: `[row N-1 data] [5-byte IDAT header] [row N filter byte] ...`

The header bytes still corrupt any file content that spans the boundary.
Row alignment helps identify WHERE boundaries fall, but doesn't eliminate the headers.

### Solution C: Multiple Small IDAT Chunks

PNG allows multiple IDAT chunks. Use chunks ≤65535 bytes each.

**Problem**: The deflate stream spans all IDATs. The internal deflate block structure
still creates 5-byte headers when data exceeds 65535 bytes.

IDAT chunk boundaries ≠ deflate block boundaries.

### Solution D: Use Non-Stored Deflate

Use dynamic Huffman that outputs bytes unchanged (like "stored" but no size limit).

**Problem**: Huffman codes can't all be 8-bit aligned. With 257 symbols needed
(256 literals + end-of-block), can't fit in 8-bit prefix-free codes.

Would require complex bit-level manipulation and wouldn't preserve byte alignment.

## Recommended Solution: A (Pad to Avoid Boundaries)

This is the simplest approach that actually works.

**Algorithm**:
```
filtered_pos = 0
for each file:
    file_header_size = 30 + name_len + extra_field
    file_content_size = deflate blocks for body
    file_total = file_header_size + file_content_size

    // Check if file would cross next boundary
    next_boundary = ((filtered_pos / 65535) + 1) * 65535
    file_end = filtered_pos + file_total

    if file_end > next_boundary:
        // Add padding to reach boundary
        padding = next_boundary - filtered_pos
        add padding bytes
        filtered_pos = next_boundary

    // Place file
    add file header and content
    filtered_pos += file_total
```

**Edge case**: File larger than 65535 bytes
- Single file can't span a boundary
- Max file size ≈ 65535 - header_size ≈ 65500 bytes per file
- For larger files, would need to split (not supported currently)

**Overhead estimate**:
- Worst case: 65535 bytes padding per file
- Average case: ~32KB padding per file if files randomly sized
- Best case: 0 padding if files naturally fit in blocks

## Questions to Resolve

1. How to handle files larger than ~65KB? (Currently: don't support them)
2. Should we constrain row_width to divide 65535 for cleaner alignment?
3. How does padding interact with row alignment for filter byte trick?
