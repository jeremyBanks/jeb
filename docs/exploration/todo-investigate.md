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

**Status**: SOLVED

**Solution implemented**: Pad files so they don't cross IDAT boundaries.

Before placing each file, we check if it would cross a 65535-byte boundary in
the filtered data. If so, we add padding to push the file past the boundary.

**Results**:
- No more total content limit
- Individual files limited to ~60KB (must fit in one IDAT block)
- Verified: 90KB polyglot (3 × 30KB files) works correctly
- 310×311 pixel PNG, all ZIP files extract with correct CRC

See `docs/exploration/idat-boundary-analysis.md` for full analysis.
