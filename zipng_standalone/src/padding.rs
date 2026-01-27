use std::ops::Range;

/// Writes `bytes` to `buffer`, padded with trailing zeroes to the next multiple
/// of `alignment`. Returns the range that `bytes` was written to in `buffer`,
/// excluding the padding.
pub fn write_aligned_pad_end(buffer: &mut Vec<u8>, bytes: &[u8], alignment: usize) -> Range<usize> {
    let index_before_data = buffer.len();

    buffer.extend_from_slice(bytes);

    let index_after_data = buffer.len();

    if index_after_data % alignment != 0 {
        let padding = alignment - (index_after_data % alignment);
        for _ in 0..padding {
            buffer.push(0);
        }
    }

    let _index_after_padding = buffer.len();

    index_before_data..index_after_data
}

/// Writes `bytes` to `buffer`, padded with leading zeroes to the next multiple
/// of `alignment`. Returns the range that `bytes` was written to in `buffer`,
/// excluding the padding.
pub fn write_aligned_pad_start(
    buffer: &mut Vec<u8>,
    bytes: &[u8],
    alignment: usize,
) -> Range<usize> {
    let index_before_padding = buffer.len();
    let unpadded_index_after_data = index_before_padding + bytes.len();
    if unpadded_index_after_data % alignment != 0 {
        let padding = alignment - (unpadded_index_after_data % alignment);
        for _ in 0..padding {
            buffer.push(0);
        }
    }
    let index_before_data = buffer.len();
    buffer.extend_from_slice(bytes);
    let index_after_data = buffer.len();
    index_before_data..index_after_data
}
