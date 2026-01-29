# Removing the 60KB Per-File Limit

## Background

The polyglot PNG+ZIP encoder currently limits individual files to
`MAX_FILE_CONTENT_SIZE = 60,000` bytes. This document explains why
the limit exists, what approaches were considered for removing it,
and what a viable implementation would look like.

## Why the Limit Exists

There are **two levels** of DEFLATE stored blocks in a zipng file:

### Inner DEFLATE (per-file compression)

Each ZIP file entry uses `compression_method = 8` (Deflate). The file's
content is encoded as a series of DEFLATE stored blocks, one per pixel
row. Each row looks like:

```
[filter byte] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi] [file data...]
```

The PNG filter byte (0x00 = None, 0x01 = Sub) doubles as the DEFLATE
block header (BFINAL + BTYPE bits). This is the core polyglot trick.

### Outer DEFLATE (IDAT zlib wrapper)

PNG requires IDAT data to be zlib-compressed. The encoder uses stored
(uncompressed) DEFLATE blocks for the outer layer too, so the filtered
pixel data appears verbatim in the file. But DEFLATE stored blocks have
a **16-bit LEN field**, limiting each block to **65,535 bytes**.

The `write_idat_stored` function splits the filtered data into 65,535-byte
chunks. At each boundary, **5 bytes** are inserted into the raw file:

```
[BFINAL|BTYPE byte] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi]
```

### The Problem

ZIP decoders read raw bytes from the file. A ZIP file entry's
`compressed_size` tells the decoder exactly how many raw bytes to read
after the local file header. If a file's inner DEFLATE stream spans an
outer block boundary, those 5 framing bytes appear in the middle of
what the ZIP decoder thinks is compressed data. The inner DEFLATE
decoder encounters unexpected bytes and fails.

The current solution: ensure every file fits entirely within one outer
DEFLATE block (65,535 bytes of filtered data), with margin for row
overhead, giving the ~60,000 byte practical limit.

The code already accounts for the cumulative outer block overhead when
computing ZIP local file header offsets:

```rust
let deflate_blocks_before = filtered_pos / IDAT_BLOCK_SIZE;
let deflate_overhead = deflate_blocks_before * 5;
let file_offset = data_offset + filtered_pos + deflate_overhead;
```

## Approaches Considered

### Approach A: Single IDAT PNG chunk (not the issue)

The encoder already writes a single IDAT PNG chunk. The 65,535 limit
comes from the outer DEFLATE stored blocks *within* that chunk, not
from PNG chunk framing. Changing IDAT chunking has no effect.

**Verdict: Not applicable.** The bottleneck is DEFLATE, not PNG chunks.

### Approach B: Single outer DEFLATE block

If the entire filtered data fit in one outer DEFLATE stored block,
there would be no boundaries to worry about. But DEFLATE stored blocks
use a 16-bit LEN field — the 65,535 byte limit is a hard format
constraint with no workaround.

**Verdict: Impossible.** DEFLATE spec constraint.

### Approach C: Use compression_method = 0 (Stored) in ZIP

If ZIP entries used stored (uncompressed) mode instead of Deflate, the
ZIP decoder would read raw bytes as the file content directly. But
those raw bytes include filter bytes, inner DEFLATE LEN/NLEN headers,
and (if spanning a boundary) the 5-byte outer framing — all of which
would be included in the extracted file.

**Verdict: Not viable.** Extracted files would contain format overhead
bytes that are not part of the actual content.

### Approach D: Hope DEFLATE decoders skip framing bytes

DEFLATE stored blocks have explicit lengths. After reading LEN bytes
the decoder expects another block header or end-of-stream. There is no
mechanism to skip arbitrary injected bytes.

**Verdict: Not viable.** No such DEFLATE feature exists.

### Approach E: Land outer boundaries in safe zones (VIABLE)

The outer DEFLATE block boundaries occur at **predictable, deterministic
positions** — every 65,535 bytes of filtered data. The 5-byte headers
they inject are already tracked by the offset calculation logic. If the
layout engine ensures those 5 bytes always land within regions that are
not part of any file's compressed content, files can span boundaries
freely.

**Verdict: Viable.** This is the recommended approach. See detailed
design below.

## Viable Approach: Safe-Zone Boundary Landing

### Core Idea

Instead of "no file may cross a boundary," the constraint becomes
"every outer DEFLATE block header must land in a safe zone." The
layout engine arranges data so that the 5-byte headers fall within
bytes that no ZIP decoder will interpret as file content.

### Safe Zones

The following regions are not part of any file's inner DEFLATE stream
and can absorb the 5-byte outer headers:

1. **Padding between files.** The encoder already inserts padding rows
   for alignment and visual spacing. These bytes are not referenced by
   any ZIP entry's compressed_size/offset.

2. **ZIP local file header extra fields.** Each local file header has a
   variable-length extra field (currently used for row alignment). The
   extra field length is controlled by the encoder. A boundary landing
   inside the extra field would not affect the file content, provided
   the extra field is structured so the injected bytes don't break the
   extra field's own TLV format. Since many ZIP tools ignore malformed
   extra fields, this is likely safe but less clean than option 1.

3. **Filename label rows.** These are cosmetic pixel rows rendered above
   each file. They are not part of any ZIP entry. A boundary here would
   cause a minor visual glitch (5 pixels of wrong color) but nothing
   structural.

4. **Terminator rows.** These contain the BFINAL=1 empty DEFLATE block
   that ends a file's stream. They are part of the file's
   compressed_size but only contain structural DEFLATE bytes (the
   terminator), not file content. However, landing an outer header here
   would corrupt the inner DEFLATE terminator, so this zone requires
   more care — the terminator would need to be restructured around the
   injected bytes.

**Recommended safe zones in priority order:** padding (1), then labels
(3), then extra fields (2). Terminator rows (4) should be avoided
unless the terminator can be split to accommodate the injection.

### Implementation Considerations

#### Boundary Position Calculation

Outer DEFLATE block boundaries occur at filtered data positions that
are multiples of 65,535. In terms of raw file bytes, the Nth boundary
is at:

```
raw_position(N) = zlib_header_size + N * (65535 + 5)
```

where the `+5` accounts for each preceding outer block header. The
filtered data position for boundary N is simply `N * 65535`.

The layout engine already converts between data positions and filtered
positions (`data_to_filtered_pos`). Extending this to predict exactly
which filtered byte offset each boundary falls on is straightforward.

#### Constraint Solver Changes

The current bin-packing layout (`assign_files_to_buckets`,
`calculate_bucket_spacing`, `build_pixel_data`) treats each 65,535-byte
outer block as an independent bucket. Files are assigned to buckets
using worst-fit decreasing, and no file may span two buckets.

The new layout must:

1. **Remove the per-bucket file size limit.** Files can be any size.

2. **Predict boundary positions** after tentative file placement. Since
   boundaries depend on total filtered data size (which depends on row
   width, which depends on total data), this may require an iterative
   approach or a two-pass layout.

3. **Insert padding at boundary positions.** When a boundary would land
   inside a file's inner DEFLATE stream, insert enough padding *before*
   that file (or between files) to shift the boundary into a safe zone.
   The padding must be a whole number of rows to maintain row alignment.

4. **Adjust compressed_size and offsets.** The 5-byte outer headers are
   not part of the filtered data, so they don't affect inner DEFLATE
   stream byte counts. But they do affect raw file offsets. The existing
   `deflate_overhead` calculation already handles this; it just needs to
   remain correct as files span multiple outer blocks.

#### Spanning Multiple Outer Blocks

When a file spans K outer blocks, K-1 boundaries fall within its
compressed data region in the raw file. Each injects 5 bytes. The ZIP
local file header's `compressed_size` refers to raw bytes, so it must
account for these injections:

```
compressed_size_raw = compressed_size_filtered + (K - 1) * 5
```

Wait — this is a critical subtlety. The ZIP decoder reads
`compressed_size` bytes starting from the offset after the local file
header. If outer DEFLATE headers are injected within that byte range,
the ZIP decoder *will* read them as part of the compressed data. **This
means the inner DEFLATE stream, as seen by the ZIP decoder, includes
those 5-byte injections.**

This breaks inner DEFLATE decoding. The safe-zone approach avoids this
by ensuring boundaries *never* land within a file's compressed region —
but that's exactly the same as the current approach (files can't cross
boundaries), just with a more flexible layout.

#### The Real Question

Can we land boundaries *outside* all files' compressed regions while
still allowing individual files larger than 60KB?

Yes, but only if files larger than 60KB are split across non-adjacent
regions of the filtered data, with boundaries landing in the gaps. This
would require:

- Splitting a large file into multiple segments, each fitting within
  one outer block
- Each segment is a valid inner DEFLATE stored block sequence
- The ZIP local file header's compressed_size covers all segments plus
  the gaps between them (including the outer DEFLATE headers in those
  gaps)

But this means the gaps (safe zones) become part of the ZIP's
compressed data byte range. The inner DEFLATE decoder would encounter
the padding bytes in those gaps and fail.

#### Revised Assessment

The safe-zone approach works **only if** the injected outer DEFLATE
header bytes are placed where the inner DEFLATE decoder expects valid
block headers. Specifically:

An inner DEFLATE stored block sequence looks like:

```
[BFINAL|BTYPE] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi] [data...]
[BFINAL|BTYPE] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi] [data...]
...
```

If we arrange the layout so that an outer header's 5 bytes land exactly
where an inner block boundary falls, the outer header bytes must form a
valid inner DEFLATE stored block header. An outer header looks like:

```
[0x00 or 0x01] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi]
```

This is *also* a valid DEFLATE stored block header (BFINAL=0 or 1,
BTYPE=00, LEN, NLEN) — as long as LEN and NLEN are complements. And
they are, because the outer encoder always writes `LEN` and `!LEN`.

**This means an outer DEFLATE block header is itself a valid inner
DEFLATE stored block header**, describing a stored block of up to
65,535 bytes.

But this is actually a problem: the inner DEFLATE decoder would
interpret the outer header as a new inner stored block with LEN bytes
of content — and then read the next LEN bytes as file data, which
would include actual file data from subsequent inner blocks. The
decoded file content would be corrupted (duplicated/shifted data).

#### The Real Viable Path

For the outer header to be harmless to the inner DEFLATE decoder, it
must look like a **zero-length stored block**: `[0x00] [0x00 0x00]
[0xFF 0xFF]`. That's 5 bytes describing an empty block with BFINAL=0.

This would require the outer DEFLATE block boundary to fall at a point
where the remaining filtered data in the current outer block is exactly
0 bytes — i.e., the boundary falls exactly at the end of a row. But
the outer block size is 65,535, and the filtered row size is
`row_width + 1`. For the boundary to align with a row ending, we need:

```
65535 mod (row_width + 1) == 0
```

This is only possible for specific row widths. For example,
`row_width + 1 = 3` (row_width=2), `5`, `9`, `15`, etc. — divisors of
65,535 = 3 × 5 × 17 × 257. The minimum row width of 68 gives
filtered_row_size = 69 = 3 × 23, and `65535 mod 69 = 18`. Not aligned.

So exact alignment is not generally achievable.

#### Alternative: Custom Outer Block Sizes

Instead of using the maximum 65,535 bytes per outer block, the encoder
could choose outer block sizes that align with row boundaries. For
example, if `filtered_row_size = 69`, use outer blocks of
`69 * 949 = 65481` bytes (the largest multiple of 69 that fits in
16 bits). Then every outer block boundary falls exactly between rows.

The 5-byte outer header would then be injected between rows. In the
raw file byte stream, this means 5 extra bytes appear between the last
byte of one row and the first byte of the next row. These 5 bytes are
*outside* the filtered pixel data — they're outer DEFLATE framing.

From the ZIP decoder's perspective: if a file spans this boundary, its
`compressed_size` bytes include those 5 framing bytes. The inner
DEFLATE decoder reads them as an inner block header. Since they
describe a stored block of `outer_block_len` bytes, this would
catastrophically misinterpret the stream.

Unless... the 5 bytes land at an inner DEFLATE block boundary AND form
a zero-length empty block. With aligned outer blocks, the boundary is
between rows. The inner DEFLATE blocks are also row-aligned (one per
row). So the boundary is between two inner blocks. If the outer header
is `[0x00] [LEN_lo LEN_hi] [NLEN_lo NLEN_hi]` where LEN is the next
outer block's size — that's not a zero-length block.

**We don't control what the outer header contains** — it's determined
by the size of the next chunk of filtered data, which is the
(potentially large) outer block size.

### Corrected Viable Path: Row-Aligned Outer Blocks + Adjusted Inner Stream

If outer blocks are row-aligned, their headers always land between
inner DEFLATE blocks (between rows). The inner DEFLATE decoder finishes
one stored block (a row of file data) and then encounters the 5-byte
outer header.

The outer header is: `[0x00] [block_len_lo block_len_hi] [nlen_lo nlen_hi]`

The inner decoder reads byte 0x00 as BFINAL=0, BTYPE=00 (stored block).
Then reads block_len as LEN. Then reads nlen as NLEN. It checks
`NLEN == !LEN` — and this will be true because the outer encoder writes
correct complements.

Now the inner decoder thinks there's a stored block of `block_len`
bytes. It reads that many bytes as literal file data. But those bytes
are the actual next rows of pixel data — which are the real inner
DEFLATE blocks (with their own headers). The decoded output would be
the raw bytes of inner block headers + file data, corrupting the file.

**This does not work either.** The outer header, while syntactically
valid DEFLATE, describes a block whose length encompasses the entire
next outer block — swallowing all the inner block structure.

### Nuclear Option: Modify the Inner Stream to Expect the Outer Headers

What if the inner DEFLATE stream is constructed so that at each outer
boundary, the inner stream has a zero-length stored block whose header
bytes happen to be the outer DEFLATE header? This requires:

- Knowing the outer block size at inner stream construction time (yes,
  we control both)
- Inserting a "ghost" inner block at the boundary point whose raw bytes
  are exactly the outer header bytes

But the outer header bytes describe a block of `outer_block_len` bytes,
not zero bytes. The inner decoder would still try to read
`outer_block_len` bytes of literal data.

**No arrangement of inner blocks can neutralize an outer header that
describes a non-zero-length block.**

### Actually Viable: Outer Zero-Length Blocks

What if we force every outer DEFLATE block to be **zero-length**?
Instead of 65,535-byte outer blocks, emit outer blocks of 0 bytes,
inserted between rows. The outer header would be:

```
[0x00] [0x00 0x00] [0xFF 0xFF]
```

This is a valid zero-length stored block. The inner DEFLATE decoder
would see it as an empty stored block (0 bytes of data) and continue
to the next inner block. **No file data is affected.**

But this means every row of filtered data is its own outer block:
`[outer header: 5 bytes] [row data: row_width+1 bytes]`. The overhead
is 5 bytes per row. For a 69-byte filtered row, that's 5/69 ≈ 7%
overhead. And the raw file grows substantially.

More critically: the ZIP local file header offsets and compressed sizes
must account for these 5-byte-per-row injections. The compressed_size
for a file spanning R rows would be:

```
compressed_size = R * (row_width + 1) + R * 5
               = R * (row_width + 6)
```

Wait — the outer zero-length blocks contain no data. The row data would
go into separate non-zero outer blocks. Let me reconsider.

Actually: one outer zero-length block *between* each row, plus one
outer block *containing* each row. Each row is in its own outer block
of size `row_width + 1`. So:

```
[outer hdr: 5B] [row 0: filtered_row_size B]
[outer hdr: 5B, zero-len] [outer hdr: 5B] [row 1: filtered_row_size B]
[outer hdr: 5B, zero-len] [outer hdr: 5B] [row 2: filtered_row_size B]
...
```

No — this is getting convoluted. The simpler version: each row is its
own outer DEFLATE block. No zero-length blocks needed. Overhead is 5
bytes per row.

The inner DEFLATE decoder never sees outer headers because... wait, it
does. The ZIP decoder reads raw file bytes. If a file starts at raw
offset X and has compressed_size S, it reads bytes X through X+S-1.
Those bytes include outer DEFLATE headers at every row boundary.

**The fundamental problem remains: ZIP reads raw bytes, and outer
DEFLATE headers exist in the raw byte stream within the file's data
region.**

## Conclusion

### Why the limit cannot be easily removed

The 60KB per-file limit is a fundamental consequence of the polyglot
design:

1. **PNG requires** the filtered pixel data to be wrapped in a zlib
   DEFLATE stream.

2. **DEFLATE stored blocks** have a 16-bit LEN field, limiting each
   block to 65,535 bytes.

3. **Outer block boundaries** inject 5 bytes of framing into the raw
   file byte stream.

4. **ZIP decoders** read raw bytes — they see the outer framing bytes
   as part of the file's compressed data.

5. **No inner DEFLATE construction** can neutralize an outer header
   that describes a non-zero-length block, because the inner decoder
   would interpret it as a large literal data block and swallow
   subsequent inner structure.

6. **Choosing specific outer block sizes** (e.g., row-aligned) ensures
   boundaries land between rows, but the outer header is still read by
   the inner DEFLATE decoder as a non-empty stored block, corrupting
   the stream.

7. **Zero-length outer blocks** between every row would work for the
   inner DEFLATE decoder (it sees empty blocks and skips them), but
   the outer headers still occupy raw file bytes within the ZIP entry's
   compressed data range, increasing compressed_size in a way that
   includes non-content bytes in the decompressed output.

The only way to fully remove the limit would be to **fundamentally
change the polyglot architecture** — for example, by abandoning stored
DEFLATE for the outer zlib layer and using actual compression (which
would destroy the polyglot property since the pixel bytes would no
longer appear verbatim in the file).

### The 60KB limit is inherent to the design

The current limit of `MAX_FILE_CONTENT_SIZE = 60,000` bytes is a
reasonable practical bound. It provides margin within the 65,535-byte
outer DEFLATE block for:

- Row overhead (filter bytes, inner DEFLATE LEN/NLEN headers)
- ZIP local file header
- Extra field padding for alignment
- Terminator rows
- Label rows

Multiple files can exceed 60KB in aggregate — only individual files
are limited. The total archive size is unbounded.
