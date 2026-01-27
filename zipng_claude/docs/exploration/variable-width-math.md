# Variable Width Calculation (EXPLORATORY)

## Goal
Calculate row width that produces approximately square images for any content size.

## Math

Let:
- W = row width (bytes) = image width (pixels at 8-bit)
- D = total data bytes to store
- Data per row = W - 4 (subtract LEN + NLEN overhead)
- Number of rows R = ceil(D / (W - 4))
- Image dimensions: W × R

For square image: W ≈ R

```
W ≈ D / (W - 4)
W × (W - 4) ≈ D
W² - 4W - D ≈ 0
W = (4 + sqrt(16 + 4D)) / 2 = 2 + sqrt(4 + D)
```

## Examples

| Content Size | Ideal Width | Approx Dimensions |
|--------------|-------------|-------------------|
| 1 KB         | 34 px       | 34 × 33          |
| 5 KB         | 73 px       | 73 × 72          |
| 10 KB        | 102 px      | 102 × 102        |
| 20 KB        | 143 px      | 143 × 143        |
| 40 KB        | 202 px      | 202 × 201        |

## Constraints

1. **Minimum width**: ~38 bytes so filter bytes don't corrupt ZIP headers
2. **Maximum width**: Limited by bit depth (row must be whole pixels)
3. **IDAT limit**: Still ~42KB total due to block boundaries

## Algorithm

```
fn calculate_optimal_width(total_data_bytes: usize) -> usize {
    // Calculate ideal square width
    let ideal = 2.0 + (4.0 + total_data_bytes as f64).sqrt();
    let width = ideal.ceil() as usize;

    // Clamp to minimum safe width
    width.max(MIN_SAFE_WIDTH)
}
```

Where MIN_SAFE_WIDTH ≈ 38 (ensures filter at position W+1 is past ZIP header).
