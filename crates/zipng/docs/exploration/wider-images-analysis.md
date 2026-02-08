# Wider Images Analysis (EXPLORATORY)

*This document synthesizes findings from parallel analysis of different approaches.*
*Status: Exploratory - not yet implemented*

## Key Findings

### Finding 1: Large Row Widths Avoid Header Corruption Entirely

For ROW_WIDTH >= 38 (with short filenames), the first filter byte lands AFTER the
entire ZIP local header, meaning no filter bytes corrupt any header fields.

| ROW_WIDTH | First Filter Position | Image Width (8-bit) | Data/Row |
|-----------|----------------------|---------------------|----------|
| 38        | 38 (past 30-byte header + 8-byte filename) | 38 px | 34 bytes |
| 44        | 44 | 44 px | 40 bytes |
| 48        | 48 | 48 px | 44 bytes |
| 64        | 64 | 64 px | 60 bytes |

**Tradeoff**: Larger rows = wider images but more overhead per small file.

### Finding 2: Extra Fields Can Absorb Filter Bytes

The ZIP extra field format allows arbitrary data after a 4-byte header.
Filter bytes landing in extra field DATA are harmless.

Strategy:
1. Size extra field so first filter lands inside its data section
2. Pad extra field so file content starts at a row boundary
3. Filter bytes (0x00/0x01) blend into the opaque extra data

Example with ROW_WIDTH=44, 10-byte filename:
```
[0]      Filter byte (PNG row start)
[1-30]   ZIP local header
[31-40]  Filename (10 bytes)
[41-44]  Extra field header (4 bytes)
[45]     FILTER BYTE (lands in extra data - safe!)
[46-89]  More extra field data + padding
[90]     Next filter = start of file content deflate block
```

### Finding 3: PNG Row Structure is Rigid

PNG strictly requires one filter byte per row. We cannot have "logical" vs "visual"
rows. The only ways to get wider visual images:

1. **Use larger ROW_WIDTH** (what we're exploring)
2. **Use lower bit depth** (packs more pixels per byte)
3. **Use indexed color with palette tricks** (hide filter artifacts)

### Finding 4: Indexed Color Can Hide Filter Byte Artifacts

If filter bytes appear as pixels (in larger-row-width scenarios), they have values
0x00 or 0x01. With indexed color mode:
- Palette entry 0 and 1 can map to any colors
- Using tRNS chunk, entries 0 and 1 can be transparent
- Filter byte "pixels" become invisible

## Promising Approaches

### Approach A: ROW_WIDTH = 64 (Simplest)

- First filter at position 64, well past any reasonable header
- 64 pixels wide at 8-bit (or 512 at 1-bit)
- 60 bytes of data per row
- No special header alignment needed
- Each file needs padding so content starts at row boundary

**Pros**: Simple, wide images, efficient for larger files
**Cons**: More overhead for small files, ~42KB limit still applies

### Approach B: ROW_WIDTH = 44 with Extra Field Absorption

- Use extra fields strategically sized to absorb filters
- 44 pixels wide at 8-bit
- More complex but works with shorter filenames
- Can calculate exact extra field size per file

**Pros**: Moderate width, flexible
**Cons**: Complex calculation, variable extra field sizes

### Approach C: Keep ROW_WIDTH = 13, Use 2-bit Color

- Current approach gives 52 pixels at 2-bit depth
- No code changes needed for width
- Just use BitDepth::TwoBit

**Pros**: No structural changes needed
**Cons**: Limited to 4 colors, still narrow at 8-bit

## Recommended Path Forward

1. **Implement configurable ROW_WIDTH** with validation
2. **Start with ROW_WIDTH = 64** as the "wide" option
3. **Calculate required padding** per file for content alignment
4. **Test with various content sizes**
5. **Consider indexed color + transparency** for hiding any artifacts

## Open Questions

1. How does ROW_WIDTH affect the 42KB IDAT limit? (Same limit, different row count)
2. Should ROW_WIDTH be user-configurable or auto-calculated based on content?
3. What's the ideal balance between width and efficiency?

## Next Steps

- [ ] Implement variable ROW_WIDTH support
- [ ] Add content-start alignment padding logic
- [ ] Test with ROW_WIDTH = 64
- [ ] Measure overhead vs current implementation
