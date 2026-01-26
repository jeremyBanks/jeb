//! Polyglot PNG+ZIP file creation.
//!
//! Creates files that are simultaneously valid PNGs and valid ZIPs,
//! where the same bytes serve as both PNG pixel data and ZIP content.
//!
//! ## Key Insight: Filter Bytes as Deflate Block Headers
//!
//! PNG filter bytes are inserted at the start of each row:
//! - Filter 0x00 (None) = deflate stored block, NOT final
//! - Filter 0x01 (Sub) = deflate stored block, FINAL
//!
//! By aligning file content to row boundaries and using the appropriate
//! filter type for each row, we get valid deflate streams!
//!
//! ## Row structure (ROW_WIDTH = 13):
//! - Filter byte (from PNG) = deflate block header
//! - 2 bytes: LEN
//! - 2 bytes: NLEN (~LEN)
//! - 9 bytes: data
//!
//! ## IDAT Block Boundaries
//!
//! IDAT uses stored deflate blocks with max 65535 bytes each. To support
//! files larger than ~42KB total, we pad so no file's content spans an
//! IDAT block boundary.

use std::collections::HashSet;
use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::png::{BitDepth, ColorMode, write_png_chunk, write_png_header, write_png_footer};

/// Row width in bytes. Each row = 1 filter + ROW_WIDTH data.
/// For deflate: filter(1) + LEN(2) + NLEN(2) + data(9) = 14 bytes per row.
const ROW_WIDTH: usize = 13;

/// Data bytes per deflate block (after LEN+NLEN overhead).
const DATA_PER_BLOCK: usize = ROW_WIDTH - 4; // = 9

/// Filtered bytes per row (filter byte + data).
const FILTERED_ROW_SIZE: usize = ROW_WIDTH + 1; // = 14

/// Maximum bytes per IDAT deflate stored block.
const IDAT_BLOCK_SIZE: usize = 65535;

/// Information about a file entry.
#[derive(Debug, Clone)]
struct FileEntry {
    name: Vec<u8>,
    body: Vec<u8>,
    crc: u32,
    header_offset: u32,
    compressed_size: u32,
}

/// Build a polyglot PNG+ZIP file.
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    _width: u32,
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8> {
    // Step 1: Build pixel data and track which rows need final block headers
    let (pixel_data, entry_infos, final_block_rows) = build_aligned_data(files);

    // Step 2: Calculate PNG dimensions
    let height = (pixel_data.len() + ROW_WIDTH - 1) / ROW_WIDTH;
    let mut padded = pixel_data.clone();
    padded.resize(height * ROW_WIDTH, 0);

    // Calculate effective pixel width
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let effective_width = (ROW_WIDTH * 8) / bits_per_pixel;

    // Step 3: Build PNG
    let mut output = Vec::new();
    write_png_header(&mut output, effective_width as u32, height as u32, bit_depth, color_mode);

    if let Some(p) = palette {
        write_png_chunk(&mut output, b"PLTE", p);
    }

    // Calculate offset where filtered pixel data starts in the file
    // PNG sig (8) + IHDR chunk (25) + PLTE if any + IDAT header (8) + zlib (2) + deflate (5)
    let plte_size = palette.map(|p| 4 + 4 + p.len() + 4).unwrap_or(0);
    let data_offset = 8 + 25 + plte_size + 8 + 2 + 5;

    // IDAT with smart filter bytes
    let filtered = add_smart_filter_bytes(&padded, ROW_WIDTH, &final_block_rows);
    write_idat_stored(&mut output, &filtered);

    write_png_footer(&mut output);

    // Step 4: Build file entries with correct offsets
    // Must account for IDAT deflate block headers every 65535 bytes
    let file_entries: Vec<FileEntry> = entry_infos
        .into_iter()
        .map(|(name, body, orig_pos, compressed_size)| {
            let crc = crc32(&body);
            // Convert original position to filtered position
            let rows_before = orig_pos / ROW_WIDTH;
            let filtered_pos = orig_pos + rows_before + 1; // +1 for initial filter

            // Account for IDAT deflate block headers (5 bytes each) every 65535 bytes
            let deflate_blocks_before = filtered_pos / 65535;
            let deflate_overhead = deflate_blocks_before * 5;

            let file_offset = data_offset + filtered_pos + deflate_overhead;
            FileEntry {
                name,
                body,
                crc,
                header_offset: file_offset as u32,
                compressed_size: compressed_size as u32,
            }
        })
        .collect();

    // Step 5: Central directory (after IEND)
    let cd_start = output.len();
    write_central_directory(&mut output, &file_entries);
    let cd_end = output.len();

    // Step 6: EOCD
    write_eocd(
        &mut output,
        file_entries.len() as u16,
        (cd_end - cd_start) as u32,
        cd_start as u32,
    );

    output
}

/// Build pixel data and return (data, entry_info, final_block_rows).
///
/// This function handles alignment at multiple levels:
/// 1. Row alignment - each file header starts at a row boundary
/// 2. Content alignment - file content starts at a row boundary
/// 3. IDAT block alignment - file content doesn't span 65535-byte boundaries
fn build_aligned_data(files: &[(&[u8], &[u8])]) -> (Vec<u8>, Vec<(Vec<u8>, Vec<u8>, usize, usize)>, HashSet<usize>) {
    let mut data = Vec::new();
    let mut entries = Vec::new();
    let mut final_block_rows = HashSet::new();

    for (name, body) in files {
        // Calculate file layout
        let header_plus_name = 28 + name.len();
        let extra_for_content_align = (ROW_WIDTH - (header_plus_name % ROW_WIDTH)) % ROW_WIDTH;
        let total_header_size = header_plus_name + extra_for_content_align;
        let header_rows = (total_header_size + ROW_WIDTH - 1) / ROW_WIDTH;

        // Calculate content size
        let num_blocks = if body.is_empty() { 1 } else { (body.len() + DATA_PER_BLOCK - 1) / DATA_PER_BLOCK };
        let compressed_size = num_blocks * FILTERED_ROW_SIZE;
        let content_rows = num_blocks;

        // Align to row boundary for header start
        let padding_to_row = (ROW_WIDTH - (data.len() % ROW_WIDTH)) % ROW_WIDTH;
        data.resize(data.len() + padding_to_row, 0);

        // Calculate where content would start and end in filtered coordinates
        let current_rows = data.len() / ROW_WIDTH;
        let content_start_row = current_rows + header_rows;
        let content_end_row = content_start_row + content_rows;

        // In filtered data: each row is FILTERED_ROW_SIZE bytes
        let content_start_filtered = content_start_row * FILTERED_ROW_SIZE;
        let content_end_filtered = content_end_row * FILTERED_ROW_SIZE;

        // Check if content would span an IDAT block boundary
        let content_start_block = content_start_filtered / IDAT_BLOCK_SIZE;
        let content_end_block = (content_end_filtered - 1) / IDAT_BLOCK_SIZE;

        if content_start_block != content_end_block && !body.is_empty() {
            // Content would span a boundary - add padding to push content past it
            // We need to add enough rows so content_start is at the next boundary
            let next_boundary = (content_start_block + 1) * IDAT_BLOCK_SIZE;
            let rows_to_add = (next_boundary - content_start_filtered + FILTERED_ROW_SIZE - 1) / FILTERED_ROW_SIZE;
            let padding_bytes = rows_to_add * ROW_WIDTH;
            data.resize(data.len() + padding_bytes, 0);
        }

        let entry_start = data.len();

        // Write header
        write_local_header(&mut data, name, body.len(), compressed_size, crc32(body), extra_for_content_align);

        // Encode body as deflate stored blocks
        let deflate_content = encode_as_deflate_blocks(body);

        // Calculate which row contains the final block
        let content_start_orig = entry_start + total_header_size;
        let last_block_orig_pos = content_start_orig + (num_blocks - 1) * ROW_WIDTH;
        let last_block_row = last_block_orig_pos / ROW_WIDTH;
        final_block_rows.insert(last_block_row);

        entries.push((name.to_vec(), body.to_vec(), entry_start, compressed_size));

        // Write deflate content (just LEN+NLEN+data per block, filter provides header)
        data.extend_from_slice(&deflate_content);
    }

    (data, entries, final_block_rows)
}

/// Encode data as deflate stored blocks.
/// Filter bytes provide block headers (0x00 or 0x01).
/// We only write: LEN(2) + NLEN(2) + data per block.
fn encode_as_deflate_blocks(body: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    if body.is_empty() {
        // Empty file: final block with 0 length
        // Filter provides 0x01 header, we write LEN=0, NLEN=0xFFFF
        result.extend_from_slice(&0_u16.to_le_bytes());
        result.extend_from_slice(&0xFFFF_u16.to_le_bytes());
        // Pad to full row width
        result.resize(ROW_WIDTH, 0);
        return result;
    }

    let chunks: Vec<&[u8]> = body.chunks(DATA_PER_BLOCK).collect();

    for chunk in chunks.iter() {
        // Filter byte provides block header (0x00 for non-final, 0x01 for final)
        // We just write LEN + NLEN + data
        let len = chunk.len() as u16;
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&len.not().to_le_bytes());
        result.extend_from_slice(chunk);

        // Pad to full row width if this is a short final block
        let block_size = 4 + chunk.len();
        if block_size < ROW_WIDTH {
            result.resize(result.len() + (ROW_WIDTH - block_size), 0);
        }
    }

    result
}

/// Write ZIP local header (28 bytes we write, filters add 2 more for 30 total).
fn write_local_header(
    data: &mut Vec<u8>,
    name: &[u8],
    uncompressed_size: usize,
    compressed_size: usize,
    crc: u32,
    extra_len: usize,
) {
    // Bytes 0-12 (we write 13 bytes; filter at position 13 provides mod_date_high)
    data.extend_from_slice(b"PK\x03\x04");
    data.extend_from_slice(&20_u16.to_le_bytes()); // version (2.0 for deflate)
    data.extend_from_slice(&0_u16.to_le_bytes());  // flags
    data.extend_from_slice(&8_u16.to_le_bytes());  // compression = deflate
    data.extend_from_slice(&0_u16.to_le_bytes());  // mod_time
    data.push(0x00); // mod_date low (high byte comes from filter = 0x00)

    // Bytes 13-25 (we write 13 bytes; filter at position 27 provides name_len_high)
    data.extend_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&(compressed_size as u32).to_le_bytes());
    data.extend_from_slice(&(uncompressed_size as u32).to_le_bytes());
    data.push((name.len() & 0xFF) as u8); // name_len low (high byte from filter = 0x00)

    // Bytes 26-27
    data.extend_from_slice(&(extra_len as u16).to_le_bytes());

    // Filename
    data.extend_from_slice(name);

    // Extra field (padding to align content)
    data.resize(data.len() + extra_len, 0);
}

/// Add PNG filter bytes, using 0x01 for rows with final deflate blocks.
fn add_smart_filter_bytes(data: &[u8], row_width: usize, final_rows: &HashSet<usize>) -> Vec<u8> {
    let mut filtered = Vec::new();

    for (i, chunk) in data.chunks(row_width).enumerate() {
        // Use filter 0x01 (Sub) for final block rows, 0x00 (None) otherwise
        let filter_type = if final_rows.contains(&i) { 0x01 } else { 0x00 };
        filtered.push(filter_type);
        filtered.extend_from_slice(chunk);
    }

    filtered
}

/// Write IDAT chunk with stored deflate.
fn write_idat_stored(buffer: &mut Vec<u8>, filtered_data: &[u8]) {
    let mut idat_content = Vec::new();

    // Zlib header
    let cmf: u8 = 0x78;
    let mut flg: u8 = 0x01;
    let check = ((cmf as u16) * 256 + (flg as u16)) % 31;
    if check != 0 {
        flg += (31 - check) as u8;
    }
    idat_content.push(cmf);
    idat_content.push(flg);

    // Single stored deflate block containing all filtered data
    for (i, chunk) in filtered_data.chunks(65535).enumerate() {
        let is_last = i == filtered_data.chunks(65535).count() - 1;
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        idat_content.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        idat_content.extend_from_slice(&(chunk.len() as u16).not().to_le_bytes());
        idat_content.extend_from_slice(chunk);
    }

    // Adler-32
    idat_content.extend_from_slice(&adler32(filtered_data).to_be_bytes());

    write_png_chunk(buffer, b"IDAT", &idat_content);
}

/// Write central directory entries.
fn write_central_directory(buffer: &mut Vec<u8>, entries: &[FileEntry]) {
    for entry in entries {
        buffer.extend_from_slice(b"PK\x01\x02");
        buffer.extend_from_slice(&20_u16.to_le_bytes()); // version made by
        buffer.extend_from_slice(&20_u16.to_le_bytes()); // version needed
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // flags
        buffer.extend_from_slice(&8_u16.to_le_bytes());  // compression = deflate
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // mod time
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // mod date
        buffer.extend_from_slice(&entry.crc.to_le_bytes());
        buffer.extend_from_slice(&entry.compressed_size.to_le_bytes());
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // extra len
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // comment len
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // disk number
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // internal attrs
        buffer.extend_from_slice(&0_u32.to_le_bytes());  // external attrs
        buffer.extend_from_slice(&entry.header_offset.to_le_bytes());
        buffer.extend_from_slice(&entry.name);
    }
}

/// Write End of Central Directory record.
fn write_eocd(buffer: &mut Vec<u8>, num_entries: u16, cd_size: u32, cd_offset: u32) {
    buffer.extend_from_slice(b"PK\x05\x06");
    buffer.extend_from_slice(&0_u16.to_le_bytes()); // disk number
    buffer.extend_from_slice(&0_u16.to_le_bytes()); // disk with CD
    buffer.extend_from_slice(&num_entries.to_le_bytes());
    buffer.extend_from_slice(&num_entries.to_le_bytes());
    buffer.extend_from_slice(&cd_size.to_le_bytes());
    buffer.extend_from_slice(&cd_offset.to_le_bytes());
    buffer.extend_from_slice(&0_u16.to_le_bytes()); // comment len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deflate_encoding() {
        // 9 bytes = exactly one block
        let data = b"123456789";
        let encoded = encode_as_deflate_blocks(data);
        // Should be LEN(2) + NLEN(2) + data(9) = 13 bytes
        assert_eq!(encoded.len(), ROW_WIDTH);
        // LEN should be 9
        assert_eq!(u16::from_le_bytes([encoded[0], encoded[1]]), 9);
    }

    #[test]
    fn test_polyglot_structure() {
        let files = vec![(b"test.txt".as_ref(), b"Hi".as_ref())];
        let result = build_polyglot(&files, 64, BitDepth::EightBit, ColorMode::Lightness, None);

        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");
        assert!(result.windows(4).any(|w| w == b"PK\x05\x06"));
    }
}
