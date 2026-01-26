# Future Investigation Items

## Color Schemes and Fonts

**Question**: Do we have different color schemes and fonts defined somewhere in this repository?

**Status**: Not yet investigated

**Context**: User asked about this during polyglot development. May be relevant for:
- Customizing polyglot image appearance
- Using indexed color palettes effectively
- Text rendering in images

## Removing the 42KB Limit

**Question**: Can we work around the IDAT 65535-byte block boundary issue?

**Status**: Not yet implemented

**Potential approaches**:
1. Pad so IDAT boundaries fall between files (not within file content)
2. Use row widths that divide 65535 evenly (e.g., 15, 17, 21...)
3. Multiple IDAT chunks with careful alignment

**Challenge**: IDAT block headers (5 bytes) get inserted every 65535 bytes of filtered
data. Any ZIP deflate stream spanning a boundary gets corrupted.
