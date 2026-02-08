# Option A: All-DEFLATE Padding (Stored Blocks Only)

This documents the previous padding/termination approach ("Option A"), which was
replaced by Option B (`03 00` fixed Huffman final block) for simplicity and
better visual output.

## Design

All bytes within `compressed_size` were valid DEFLATE using only stored blocks.
Padding after file content was filled with repeating empty stored blocks:
`00 00 00 FF FF` (BFINAL=0, BTYPE=00, LEN=0, NLEN=0xFFFF).

## Bridge Patterns

When padding wasn't divisible by 5, the remainder bytes "bridged" into a
terminator row. Five patterns handled R=0..4:

| R | Padding bytes            | Terminator row starts with       |
|---|--------------------------|----------------------------------|
| 0 | (all complete blocks)    | `00 00 FF FF 01 00 00 FF FF`     |
| 1 | `00`                     | `00 FF FF 01 00 00 FF FF`        |
| 2 | `00 00`                  | `FF FF 01 00 00 FF FF`           |
| 3 | `02 00 00`               | `FF FF 01 00 00 FF FF`           |
| 4 | `02 08 00 00`            | `FF FF 01 00 00 FF FF`           |

R=3 used a non-final fixed Huffman block (`02 00`) to consume 2 bytes, then
started a stored block. R=4 used two fixed Huffman blocks (`02 08 00`) then a
stored block.

## Known Bug

R=3 had an incorrect `compressed_size` calculation. The terminator row needed 8
bytes of meaningful DEFLATE data (not 7), causing the compressed size to be 1
byte too short. This was never fixed before Option A was replaced.

## Visual Problem

The `FF` bytes in empty stored blocks (`00 00 00 FF FF`) created bright white
pixels in indexed-color images (palette index 255) and bright colored pixels in
RGBA images. This was visually distracting in the padding areas of the image.

## Complexity

The approach required 8-10 code paths:
- 5 bridge byte patterns (R=0..4)
- 5 terminator row layouts
- Bit-level reasoning for R=3 and R=4 (fixed Huffman encoding)
- Separate `compressed_size` formulas for embedded vs. separate terminator

## Why Option B Was Chosen

Option B (`03 00` fixed Huffman final block) replaced Option A because:

1. **Simpler**: Only 3 code paths (padding >= 2, == 1, == 0) instead of 5+
2. **Better visuals**: `0x03` is nearly invisible (brightness 3/255), no `0xFF` artifacts
3. **Correct**: No known bugs in compressed_size calculation
4. **Acceptable compatibility**: Fixed Huffman blocks are universally supported
   by all DEFLATE decoders; the only "risk" is theoretical (some extremely
   minimal decoder that only handles stored blocks), which doesn't exist in
   practice
