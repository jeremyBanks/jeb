# Polyglot PNG+ZIP Architecture

This document describes the final architecture of the polyglot PNG+ZIP implementation.

## Overview

A polyglot file is simultaneously a valid PNG image and a valid ZIP archive. The same
bytes serve as both PNG pixel data and ZIP file contents.

## File Structure

```
┌─────────────────────────────────────┐
│ PNG Signature (8 bytes)             │
├─────────────────────────────────────┤
│ IHDR Chunk (image dimensions)       │
├─────────────────────────────────────┤
│ PLTE Chunk (optional palette)       │
├─────────────────────────────────────┤
│ IDAT Chunk (pixel data)             │  ← Contains ZIP local headers + file contents
│   ┌─────────────────────────────┐   │
│   │ Zlib header                 │   │
│   │ Deflate stored blocks       │   │
│   │   (filtered pixel data)     │   │
│   │ Adler-32 checksum           │   │
│   └─────────────────────────────┘   │
├─────────────────────────────────────┤
│ IEND Chunk (PNG end marker)         │
├─────────────────────────────────────┤
│ ZIP Central Directory               │  ← File index (after PNG ends)
├─────────────────────────────────────┤
│ ZIP End of Central Directory        │
└─────────────────────────────────────┘
```

## What's In The Pixel Data?

The IDAT chunk contains **uncompressed** (stored deflate) data that includes:

1. **ZIP Local File Headers** - For each file:
   - Signature (`PK\x03\x04`)
   - File metadata (size, CRC-32, modification time)
   - Filename
   - Extra field (used for alignment padding)

2. **File Contents** - Actual file data encoded as deflate stored blocks:
   - Each PNG row = one deflate block
   - Filter byte (0x00 or 0x01) serves as deflate block header
   - LEN + NLEN + data bytes per row

3. **Alignment Padding** - Between files and at row boundaries

**Recovery**: File contents could theoretically be recovered from pixel data alone,
without the central directory, since local headers are self-describing.

## The Filter Byte Trick

PNG prepends a filter byte to each row. We exploit this:
- `0x00` (None filter) = deflate stored block, NOT final
- `0x01` (Sub filter) = deflate stored block, FINAL (marks end of file content)

This makes PNG filter bytes serve double duty as ZIP deflate block headers.

## Compression

Both layers use **uncompressed** (stored) deflate:
- ZIP file contents: stored deflate blocks
- PNG IDAT: stored deflate blocks wrapping the filtered pixel data

This preserves exact byte alignment needed for the polyglot trick.

## Variable Row Width

Row width is calculated for approximately square images:
- Formula: `width = floor(sqrt(actual_data_size))`
- Minimum: 40 bytes (ensures filter bytes don't corrupt ZIP headers)
- Maximum: ~205 bytes (limited by 42KB content cap)

**Two-pass approach**: Build once to measure actual size, then rebuild with optimal width.

**Dimension guarantee**: Height ≥ Width (portrait/square orientation)

## Current Limitations

### 42KB Content Limit

IDAT deflate blocks have a maximum size of 65535 bytes. When filtered data exceeds
this, IDAT inserts 5-byte block headers that corrupt any ZIP data spanning boundaries.

This limits total content to ~42KB.

**Potential workarounds** (not yet implemented):
- Pad so IDAT boundaries fall between files
- Use row widths that divide 65535 evenly

### Minimum Row Width

Row width must be ≥ 40 bytes so that filter bytes (at positions 0, W+1, 2W+2, ...)
don't land inside the 30-byte ZIP local file header.

## Examples

| Content Size | Image Dimensions | Aspect Ratio |
|--------------|------------------|--------------|
| 124 bytes    | 160 × 8          | Small content, min width |
| 32 KB        | 185 × 186        | ~1:1 (square) |
| 38 KB        | 201 × 202        | ~1:1 (square) |
