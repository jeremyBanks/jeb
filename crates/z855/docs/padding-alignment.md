# Padding Alignment for Long Passthrough (`|` escape)

> **Note:** This document describes an extension to the `|` escape format. This is NOT backward compatible with the original format.

## Overview

When a long passthrough (`|` escape) has 2 or more bytes of padding space, the encoder can choose where to position the raw data within that space. An **offset** prefix is added to indicate how many padding bytes precede the raw data.

### Original Structure (padding ≤ 1 byte)
```
[length prefix][|][raw bytes][padding][|]
```

### Extended Structure (padding ≥ 2 bytes)
```
[offset prefix][length prefix][|][padding before][raw bytes][padding after][|]
```

## Why Alignment Matters

Aligning the raw data to meaningful boundaries improves human readability when inspecting encoded output. Someone correlating encoded data with original input benefits from predictable alignment.

## The Offset Prefix

### Encoding

The offset uses the same base-42 variable-length encoding as the length:
- Values 0-41: terminal digit
- Values 42-83: continuation digit (subtract 42, continue reading)

Both offset and length are self-terminating, so the decoder can distinguish them.

### Decoding

Reading backwards from `|`:
1. Read length (base-42, self-terminating)
2. Read offset (base-42, self-terminating)

**Example:** offset=3, length=10
```
[Z85 char for 3][Z85 char for 10][|]
```
Decoder reads backwards: value 10 (terminal) → length=10. Then value 3 (terminal) → offset=3.

**Example:** offset=5, length=100 (100 = 2×42 + 16)
```
[Z85 char for 5][Z85 char for 2][Z85 char for 58][|]
```
Decoder reads backwards: 58 (≥42, continuation, contributes 16), 2 (<42, terminal) → length=100. Then 5 (terminal) → offset=5.

### When Offset is Omitted

When padding space is 0 or 1 byte, there's no flexibility in positioning, so offset is **not encoded**. The decoder assumes offset=0 (raw data starts immediately after `|`).

**Threshold rule:** Offset is only encoded when `padding_available >= 2`.

## Alignment Selection Algorithm

The encoder chooses the offset that maximizes alignment using a 4-tuple sort key based on bit-reversal.

### Coordinate Systems

For a given offset choice:
- **Input coordinates**: byte positions in the original input data
- **Output coordinates**: character positions in the encoded output

### Sort Key Construction

For each candidate offset:
1. Compute `input_start` and `input_end` (byte positions of raw data in input)
2. Compute `output_start` and `output_end` (character positions in output)
3. Build sort key:

```
sort_key = (
    min(bit_reverse(input_start), bit_reverse(input_end)),
    max(bit_reverse(input_start), bit_reverse(input_end)),
    min(bit_reverse(output_start), bit_reverse(output_end)),
    max(bit_reverse(output_start), bit_reverse(output_end))
)
```

4. Compare sort keys lexicographically; **lowest wins**

### Why This Order?

- **Primary**: Input byte alignment (both endpoints). Optimizes for readers who care about original data structure.
- **Tiebreaker**: Output character alignment (both endpoints). When input alignment ties, prefer prettier output positions.

### Bit Reversal

Bit reversal makes positions with trailing zeros (aligned to power-of-2 boundaries) sort first:
- Position 0: `0b...0000` → reversed has leading zeros → small value
- Position 4: `0b...0100` → reversed still has leading zeros → small value
- Position 3: `0b...0011` → reversed has leading ones → larger value

### Example

Available padding: 4 characters. Raw data: 20 bytes starting at input position 100.

Candidate offsets: 0, 1, 2, 3, 4

For offset=0:
- input_start=100, input_end=119
- output_start=X, output_end=X+19 (depends on overall position)

For offset=2:
- Same input positions (data doesn't move in input space)
- output_start=X+2, output_end=X+21

Wait—the input positions don't change with offset. The raw data is the same bytes regardless of where we position them in the output. So input_start/input_end are fixed.

**Correction**: The alignment optimization is about where the encoded representation sits in the output stream, not about which input bytes we're encoding. The input positions are fixed by which bytes are safe. The output positions vary with offset choice.

So the sort key simplifies to:
```
sort_key = (
    min(bit_reverse(input_start), bit_reverse(input_end)),   # fixed
    max(bit_reverse(input_start), bit_reverse(input_end)),   # fixed
    min(bit_reverse(output_start), bit_reverse(output_end)), # varies with offset
    max(bit_reverse(output_start), bit_reverse(output_end))  # varies with offset
)
```

Since input coordinates are fixed, they provide a constant prefix. The tiebreaking happens entirely on output coordinates. But the input prefix ensures we're consistent with the ranking used elsewhere in the encoder.

### Mapping Output Position to Input Position

Since output characters and input bytes don't align 1:1 (4 bytes → 5 chars), we need to map output positions back to "corresponding" input positions for ranking.

For an output position `out_pos`, the corresponding input position is:
```
in_pos = floor(out_pos * 4 / 5)
```

Multiple output positions may map to the same input position (collisions). The tiebreaker resolves these.

### Full Algorithm for Offset Selection

For each candidate offset in `0..=max_padding`:
1. Compute `output_start = base_output_pos + offset`
2. Compute `output_end = output_start + raw_len - 1`
3. Map to input: `input_start = floor(output_start * 4 / 5)`
4. Map to input: `input_end = floor(output_end * 4 / 5)`
5. Build sort key (4-tuple)
6. Pick offset with lexicographically smallest key

## Decoder Behavior

1. Read length backwards from `|`
2. If padding_available >= 2: read offset backwards
3. Else: offset = 0
4. Skip `offset` padding characters
5. Read `length` raw bytes
6. Skip remaining padding to final `|`

## Backward Compatibility

This format is **NOT backward compatible**. Old decoders will misinterpret the offset as part of the length, producing incorrect results.

Deployments must ensure all decoders are updated before encoders start producing offset-enabled output.
