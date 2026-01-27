# Polyglot PNG+ZIP Constraints Analysis

This document describes all constraints affecting the polyglot PNG+ZIP implementation,
with the goal of understanding how to achieve more flexible (and more square) image dimensions.

## Overview

The polyglot creates files that are simultaneously valid PNGs and valid ZIPs, where the
same bytes serve as both PNG pixel data and ZIP file content. The key trick is using
PNG filter bytes as deflate block headers.

## Constraint Categories

### 1. PNG Filter Byte Constraints

PNG prepends a filter byte (0x00-0x04) to each row of pixel data before compression.
We exploit this by using:
- `0x00` (None filter) = deflate stored block, NOT final
- `0x01` (Sub filter) = deflate stored block, FINAL

**Constraint**: Filter bytes appear at the start of each row in the filtered data stream.
Their positions are: 0, ROW_WIDTH+1, 2*(ROW_WIDTH+1), etc.

### 2. Deflate Stored Block Structure

Each deflate stored block has the format:
```
[header: 1 byte] [LEN: 2 bytes LE] [NLEN: 2 bytes LE] [data: LEN bytes]
```

Where:
- header bit 0 = BFINAL (1 if last block)
- header bits 1-2 = BTYPE (00 = stored)
- NLEN must equal ~LEN (one's complement)

**Constraint**: If filter byte is the header, we need LEN + NLEN + data to fill the rest
of the row. With ROW_WIDTH = W:
- LEN (2 bytes) + NLEN (2 bytes) + data = W bytes
- So data per block = W - 4 bytes

### 3. ZIP Local File Header Structure

The ZIP local file header is 30 bytes minimum:
```
Offset  Size  Field
0       4     Signature (PK\x03\x04)
4       2     Version needed
6       2     Flags
8       2     Compression method
10      2     Mod time
12      2     Mod date
14      4     CRC-32
18      4     Compressed size
22      4     Uncompressed size
26      2     Filename length
28      2     Extra field length
30      var   Filename
30+n    var   Extra field
```

**Critical Constraint**: When filter bytes are inserted into this structure, they become
part of the ZIP header data. Currently we exploit this:
- Filter at position 13 (after first row) provides the HIGH BYTE of mod_date
- Filter at position 27 provides the HIGH BYTE of filename_length

This only works if:
- ROW_WIDTH + 1 = 14 (so filter at position 14 in filtered stream = position 13 in data)
- Wait, let me recalculate...

Actually, the relationship is:
- Original data position P maps to filtered position P + floor(P / ROW_WIDTH) + 1
- Filtered position F maps to original position F - floor(F / (ROW_WIDTH+1)) - 1

With ROW_WIDTH = 13:
- Row 0: filter at filtered pos 0, data at filtered pos 1-13
- Row 1: filter at filtered pos 14, data at filtered pos 15-27
- Row 2: filter at filtered pos 28, data at filtered pos 29-41

So in the original (unfiltered) data:
- Positions 0-12 become filtered positions 1-13 (row 0)
- Positions 13-25 become filtered positions 15-27 (row 1)
- Positions 26-38 become filtered positions 29-41 (row 2)

The ZIP header in original data:
- mod_date at positions 12-13: becomes filtered positions 13, 15
- filename_length at positions 26-27: becomes filtered positions 28, 29

**The filter byte at filtered position 14 lands BETWEEN original positions 12 and 13!**
**The filter byte at filtered position 28 lands BETWEEN original positions 25 and 26!**

This means filter bytes are INSERTED into the ZIP header, not replacing bytes. The ZIP
reader sees the filtered stream (with filter bytes), so we must account for this.

### 4. Current Header Alignment (ROW_WIDTH = 13)

With ROW_WIDTH = 13, the ZIP local header gets filter bytes inserted at:
- After original byte 12 (between mod_time and mod_date)
- After original byte 25 (just before filename_length)

The current implementation handles this by:
- Writing only the LOW byte of mod_date (the HIGH byte comes from filter = 0x00)
- Writing only the LOW byte of filename_length (HIGH byte from filter = 0x00)

This limits filenames to 255 bytes and dates to low values.

### 5. IDAT Deflate Block Size Constraint

PNG's IDAT chunk contains zlib-compressed data. For "stored" compression, the maximum
deflate block size is 65535 bytes. When filtered pixel data exceeds this, IDAT must
use multiple deflate blocks, inserting 5-byte headers every 65535 bytes.

**Constraint**: These inserted headers corrupt any ZIP data that spans the boundary.
Current limit: ~42KB total content.

### 6. ZIP Compressed Content Structure

Each file's "compressed" content (using deflate method 8) must be a valid deflate stream.
We construct this using filter bytes as block headers:
- Each row = one deflate stored block
- Filter byte = block header (0x00 or 0x01)
- Row data = LEN + NLEN + actual data

**Constraint**: The compressed_size field must account for filter bytes, since they're
part of the deflate stream from ZIP's perspective.

### 7. Image Dimension Constraints

Current calculation:
```rust
effective_width = (ROW_WIDTH * 8) / bits_per_pixel
height = total_rows
```

With ROW_WIDTH = 13:
- 1-bit: 104 pixels wide
- 2-bit: 52 pixels wide
- 4-bit: 26 pixels wide
- 8-bit: 13 pixels wide

**Problem**: Images are very narrow and tall (13×N for 8-bit).

## Potential Solutions for Wider Images

### Option A: Larger ROW_WIDTH with Adjusted Header Alignment

Use a larger ROW_WIDTH (e.g., 26, 39, 52) and carefully place ZIP header fields so
filter byte insertions land in acceptable positions (padding, reserved fields, etc.).

Challenges:
- Must find row widths where filter bytes don't corrupt critical ZIP fields
- May need to restructure the local header layout

### Option B: Multiple Logical Rows per Visual Row

Keep ROW_WIDTH = 13 for the deflate/ZIP structure, but treat multiple logical rows
as one visual row in the PNG.

Example: 4 logical rows = 1 visual row of 52 bytes = 52 pixels at 8-bit

Challenges:
- Filter bytes (4 of them) would appear in the visual row
- These would show as specific pixel values (0x00 or 0x01)
- Might be acceptable as visual artifacts

### Option C: Use PNG Interlacing

Adam7 interlacing reorders pixels. Might allow different effective layouts.

Challenges:
- Complex interaction with filter bytes
- May not actually help with the fundamental constraints

### Option D: Different Compression Method

Use compression method 0 (stored) in ZIP instead of 8 (deflate). Then filter bytes
don't need to be valid deflate headers.

Challenges:
- Loses the elegant filter-as-header trick
- Would need a different approach entirely

### Option E: Variable Row Width Based on Content

Use different row widths for different parts:
- Header section: ROW_WIDTH = 13 (for ZIP header alignment)
- Content section: larger ROW_WIDTH (for wider pixels)

Challenges:
- PNG requires consistent row width
- Would need padding/alignment at transitions

## Questions to Resolve

1. What row widths allow filter bytes to land in "safe" positions in the ZIP header?
2. Can we use ZIP extra fields or padding to absorb unwanted filter bytes?
3. Is Option B (multiple logical rows per visual row) visually acceptable?
4. What's the minimum acceptable aspect ratio for the images?
5. Can we solve the 42KB limit while also addressing width?

## Current Implementation Summary

- ROW_WIDTH = 13 (fixed for ZIP header alignment)
- FILTERED_ROW_SIZE = 14
- DATA_PER_BLOCK = 9 bytes of actual file content per row
- Image width: 13-104 pixels depending on bit depth
- Image height: varies with content size
- Maximum content: ~42KB (single IDAT block)
