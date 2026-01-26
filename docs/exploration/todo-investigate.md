# Future Investigation Items

## Color Schemes and Fonts

**Question**: Do we have different color schemes and fonts defined somewhere in this repository?

**Status**: Investigated

**Findings**:

### Fonts (`src/font.rs`)
6 built-in bitmap fonts via the `Font` trait:
- `Micro3pt` (3×3 pixels) - **has glyphs defined**, inspired by u/Udzu's Unicase Micro
- `Mini5pt` (3×5 pixels) - **has glyphs defined**, inspired by u/Udzu's Mini
- `Slab9pt` (9×12) - empty, inspired by Susan Kare's Toronto
- `Sans9pt` (9×12) - empty, inspired by Susan Kare's Chicago
- `Mono9pt` (9×12) - empty, inspired by Susan Kare's Monaco
- `Serif9pt` (9×12) - empty, inspired by Susan Kare's New York

The 9pt fonts have structure but no glyphs yet.

### Color Modes (`src/png.rs`)
5 PNG color modes via `ColorMode` enum:
- `Lightness` (grayscale, 1 sample/pixel)
- `RedGreenBlue` (RGB, 3 samples/pixel)
- `Indexed` (palette, 1 sample/pixel)
- `LightnessAlpha` (grayscale+alpha, 2 samples/pixel)
- `RedGreenBlueAlpha` (RGBA, 4 samples/pixel)

### Bit Depths (`src/png.rs`)
- 1, 2, 4, 8, or 16 bits per sample

No predefined color schemes/palettes found - palettes are passed in at runtime.

## Removing the 42KB Limit

**Question**: Can we work around the IDAT 65535-byte block boundary issue?

**Status**: Not yet implemented

**Potential approaches**:
1. Pad so IDAT boundaries fall between files (not within file content)
2. Use row widths that divide 65535 evenly (e.g., 15, 17, 21...)
3. Multiple IDAT chunks with careful alignment

**Challenge**: IDAT block headers (5 bytes) get inserted every 65535 bytes of filtered
data. Any ZIP deflate stream spanning a boundary gets corrupted.
