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
//! ## Variable Row Width
//!
//! Row width is calculated to produce approximately square images:
//! - width ≈ sqrt(total_data) for square proportions
//! - Minimum width of 40 ensures filter bytes don't corrupt ZIP headers
//! - Height >= width is preferred (portrait/square orientation)
//!
//! ## IDAT Block Boundaries
//!
//! IDAT uses stored deflate blocks with max 65535 bytes each. Files are
//! automatically padded to avoid crossing block boundaries, allowing
//! unlimited total content (individual files limited to ~60KB each).

use std::collections::HashSet;
use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::io::{OutputBuffer, output_buffer};
use crate::png::write_png::{write_png_header, write_png_chunk, write_png_footer};

// Re-export types from png module with compatibility aliases
pub use crate::png::{BitDepth, ColorType};
pub use crate::png::BitDepth::*;
pub use crate::png::ColorType::*;

/// Alias for backward compatibility - ColorMode is now ColorType
pub type ColorMode = ColorType;

/// Alias for backward compatibility - Lightness is now Luminance
pub const Lightness: ColorType = ColorType::Luminance;
pub const LightnessAlpha: ColorType = ColorType::LuminanceAlpha;

/// Minimum row width to ensure filter bytes don't land in ZIP headers.
/// ZIP local header is 30 bytes + filename, so 40 gives safe margin.
const MIN_ROW_WIDTH: usize = 40;

/// Maximum bytes per IDAT deflate stored block.
const IDAT_BLOCK_SIZE: usize = 65535;

/// Maximum size for a single file's compressed content.
/// Must fit within one IDAT block (65535 bytes of filtered data).
pub const MAX_FILE_CONTENT_SIZE: usize = 60_000;

/// Estimate total data size for width calculation.
fn estimate_total_size(files: &[(&[u8], &[u8])]) -> usize {
    let mut total = 0;
    for (name, body) in files {
        // ZIP header: 30 bytes + name length + padding estimate
        total += 30 + name.len() + 50;
        // File content + deflate overhead (~4 bytes per 36 bytes)
        total += body.len();
        total += (body.len() / 36 + 1) * 4;
    }
    // Minimum reasonable size
    total.max(100)
}

/// Calculate optimal row width for approximately square images.
/// Returns width such that height >= width (portrait/square orientation).
fn calculate_row_width(total_data_estimate: usize) -> usize {
    // For height >= width, we need: total_data / width >= width
    // Therefore: width <= sqrt(total_data)
    // Using floor ensures height >= width
    let ideal = (total_data_estimate as f64).sqrt();
    let width = ideal.floor() as usize;

    // Clamp to minimum safe width
    width.max(MIN_ROW_WIDTH)
}

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
    // Two-pass approach for optimal dimensions:
    // Pass 1: Build with estimated width to get actual data size
    // Pass 2: Rebuild with width calculated from actual size

    // Pass 1: Use estimate for initial build
    let estimated_size = estimate_total_size(files);
    let initial_width = calculate_row_width(estimated_size);
    let (initial_data, _, _) = build_aligned_data(files, initial_width);

    // Pass 2: Calculate optimal width from actual size, rebuild
    let actual_size = initial_data.len();
    let row_width = calculate_row_width(actual_size);
    let (pixel_data, entry_infos, final_block_rows) = build_aligned_data(files, row_width);

    // Step 2: Calculate PNG dimensions
    let height = if pixel_data.is_empty() { 1 } else { (pixel_data.len() + row_width - 1) / row_width };
    let mut padded = pixel_data.clone();
    padded.resize(height * row_width, 0);

    // Calculate effective pixel width based on bit depth
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let effective_width = (row_width * 8) / bits_per_pixel;

    // Step 3: Build PNG
    let mut output = output_buffer();
    write_png_header(&mut output, effective_width as u32, height as u32, bit_depth, color_mode);

    if let Some(p) = palette {
        let plte_data = OutputBuffer::without_tag(p);
        write_png_chunk(&mut output, b"PLTE", &plte_data);
    }

    // Calculate offset where filtered pixel data starts in the file
    // PNG sig (8) + IHDR chunk (25) + PLTE if any + IDAT header (8) + zlib (2) + deflate (5)
    let plte_size = palette.map(|p| 4 + 4 + p.len() + 4).unwrap_or(0);
    let data_offset = 8 + 25 + plte_size + 8 + 2 + 5;

    // IDAT with smart filter bytes
    let filtered = add_smart_filter_bytes(&padded, row_width, &final_block_rows);
    write_idat_stored(&mut output, &filtered);

    write_png_footer(&mut output);

    // Step 4: Build file entries with correct offsets
    let file_entries: Vec<FileEntry> = entry_infos
        .into_iter()
        .map(|(name, body, orig_pos, compressed_size)| {
            let crc = crc32(&body);
            // Convert original position to filtered position
            let rows_before = orig_pos / row_width;
            let filtered_pos = orig_pos + rows_before + 1; // +1 for initial filter

            // Account for IDAT deflate block headers (5 bytes each) every 65535 bytes
            let deflate_blocks_before = filtered_pos / IDAT_BLOCK_SIZE;
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

    output.into_bytes()
}

/// Convert data position to filtered position (accounts for filter bytes).
fn data_to_filtered_pos(data_pos: usize, row_width: usize) -> usize {
    let row = data_pos / row_width;
    let col = data_pos % row_width;
    // Each row has a filter byte prefix, so row N starts at filtered position N * (row_width + 1)
    // Data within the row is at filtered position row_start + 1 + col
    row * (row_width + 1) + 1 + col
}

/// Check if a range of filtered positions crosses an IDAT block boundary.
fn crosses_idat_boundary(start: usize, end: usize) -> bool {
    // IDAT boundaries are at positions 65535, 131070, 196605, ...
    let start_block = start / IDAT_BLOCK_SIZE;
    let end_block = (end.saturating_sub(1)) / IDAT_BLOCK_SIZE;
    start_block != end_block
}

/// Find the next row-aligned data position that starts after a filtered boundary.
fn next_boundary_aligned_pos(current_data_pos: usize, row_width: usize) -> usize {
    let filtered_row_size = row_width + 1;
    let current_filtered = data_to_filtered_pos(current_data_pos, row_width);
    let current_block = current_filtered / IDAT_BLOCK_SIZE;
    let next_boundary = (current_block + 1) * IDAT_BLOCK_SIZE;

    // Find the row that starts at or after the next boundary
    // Row N starts at filtered position N * filtered_row_size
    // We need N * filtered_row_size >= next_boundary
    let target_row = (next_boundary + filtered_row_size - 1) / filtered_row_size;

    // Return the data position for the start of that row
    target_row * row_width
}

/// Build pixel data with variable row width and IDAT boundary handling.
fn build_aligned_data(
    files: &[(&[u8], &[u8])],
    row_width: usize,
) -> (Vec<u8>, Vec<(Vec<u8>, Vec<u8>, usize, usize)>, HashSet<usize>) {
    let data_per_block = row_width - 4;
    let filtered_row_size = row_width + 1;

    let mut data = Vec::new();
    let mut entries = Vec::new();
    let mut final_block_rows = HashSet::new();

    for (name, body) in files {
        // Calculate file size before placing it
        let header_size = 30 + name.len();
        let bytes_for_alignment = header_size % row_width;
        let extra_len_estimate = if bytes_for_alignment == 0 { 0 } else { row_width - bytes_for_alignment };
        let num_blocks = if body.is_empty() { 1 } else { (body.len() + data_per_block - 1) / data_per_block };
        let file_data_size = header_size + extra_len_estimate + num_blocks * row_width;

        // Align to row boundary
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        let mut target_pos = data.len() + padding_to_row;

        // Check if file would cross an IDAT boundary
        let filtered_start = data_to_filtered_pos(target_pos, row_width);
        let filtered_end = data_to_filtered_pos(target_pos + file_data_size, row_width);

        if crosses_idat_boundary(filtered_start, filtered_end) {
            // File would cross a boundary; push it to start after the boundary
            target_pos = next_boundary_aligned_pos(target_pos, row_width);
        }

        // Pad to the target position
        data.resize(target_pos, 0);
        let entry_start = data.len();

        // Recalculate extra field size (may differ if position changed)
        let bytes_used = header_size % row_width;
        let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };

        // Calculate compressed size (with filter bytes)
        let compressed_size = num_blocks * filtered_row_size;

        // Calculate which row contains the final block
        let content_start = entry_start + header_size + extra_len;
        let last_block_pos = content_start + (num_blocks - 1) * row_width;
        let last_block_row = last_block_pos / row_width;
        final_block_rows.insert(last_block_row);

        entries.push((name.to_vec(), body.to_vec(), entry_start, compressed_size));

        // Write standard ZIP local file header (30 bytes)
        write_local_header(&mut data, name, body.len(), compressed_size, crc32(body), extra_len);

        // Encode and write deflate content
        let deflate_content = encode_as_deflate_blocks(body, row_width);
        data.extend_from_slice(&deflate_content);
    }

    (data, entries, final_block_rows)
}

/// Encode data as deflate stored blocks with variable row width.
fn encode_as_deflate_blocks(body: &[u8], row_width: usize) -> Vec<u8> {
    let data_per_block = row_width - 4;
    let mut result = Vec::new();

    if body.is_empty() {
        // Empty file: final block with 0 length
        result.extend_from_slice(&0_u16.to_le_bytes());
        result.extend_from_slice(&0xFFFF_u16.to_le_bytes());
        result.resize(row_width, 0);
        return result;
    }

    for chunk in body.chunks(data_per_block) {
        let len = chunk.len() as u16;
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&len.not().to_le_bytes());
        result.extend_from_slice(chunk);

        // Pad to full row width
        let block_size = 4 + chunk.len();
        if block_size < row_width {
            result.resize(result.len() + (row_width - block_size), 0);
        }
    }

    result
}

/// Write standard ZIP local file header (30 bytes + name + extra).
fn write_local_header(
    data: &mut Vec<u8>,
    name: &[u8],
    uncompressed_size: usize,
    compressed_size: usize,
    crc: u32,
    extra_len: usize,
) {
    // Standard 30-byte ZIP local file header
    data.extend_from_slice(b"PK\x03\x04");                           // 0-3: signature
    data.extend_from_slice(&20_u16.to_le_bytes());                   // 4-5: version needed
    data.extend_from_slice(&0_u16.to_le_bytes());                    // 6-7: flags
    data.extend_from_slice(&8_u16.to_le_bytes());                    // 8-9: compression (deflate)
    data.extend_from_slice(&0_u16.to_le_bytes());                    // 10-11: mod time
    data.extend_from_slice(&0_u16.to_le_bytes());                    // 12-13: mod date
    data.extend_from_slice(&crc.to_le_bytes());                      // 14-17: CRC-32
    data.extend_from_slice(&(compressed_size as u32).to_le_bytes()); // 18-21
    data.extend_from_slice(&(uncompressed_size as u32).to_le_bytes()); // 22-25
    data.extend_from_slice(&(name.len() as u16).to_le_bytes());      // 26-27: name length
    data.extend_from_slice(&(extra_len as u16).to_le_bytes());       // 28-29: extra length
    data.extend_from_slice(name);                                     // 30+: filename

    // Extra field for alignment padding
    if extra_len > 0 {
        if extra_len >= 4 {
            // Valid extra field structure: ID + size + data
            data.extend_from_slice(&0x0000_u16.to_le_bytes()); // header ID
            data.extend_from_slice(&((extra_len - 4) as u16).to_le_bytes()); // data size
            data.resize(data.len() + extra_len - 4, 0); // data (zeros)
        } else {
            // Just padding bytes
            data.resize(data.len() + extra_len, 0);
        }
    }
}

/// Add PNG filter bytes, using 0x01 for rows with final deflate blocks.
fn add_smart_filter_bytes(data: &[u8], row_width: usize, final_rows: &HashSet<usize>) -> Vec<u8> {
    let mut filtered = Vec::new();

    for (i, chunk) in data.chunks(row_width).enumerate() {
        let filter_type = if final_rows.contains(&i) { 0x01 } else { 0x00 };
        filtered.push(filter_type);
        filtered.extend_from_slice(chunk);
    }

    filtered
}

/// Write IDAT chunk with stored deflate.
fn write_idat_stored(buffer: &mut OutputBuffer, filtered_data: &[u8]) {
    let mut idat_content = output_buffer();

    // Zlib header
    let cmf: u8 = 0x78;
    let mut flg: u8 = 0x01;
    let check = ((cmf as u16) * 256 + (flg as u16)) % 31;
    if check != 0 {
        flg += (31 - check) as u8;
    }
    idat_content.push(cmf);
    idat_content.push(flg);

    // Stored deflate blocks (max 65535 bytes each)
    for (i, chunk) in filtered_data.chunks(IDAT_BLOCK_SIZE).enumerate() {
        let is_last = i == filtered_data.chunks(IDAT_BLOCK_SIZE).count() - 1;
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        idat_content += &(chunk.len() as u16).to_le_bytes();
        idat_content += &(chunk.len() as u16).not().to_le_bytes();
        idat_content += chunk;
    }

    // Adler-32
    idat_content += &adler32(filtered_data).to_be_bytes();

    write_png_chunk(buffer, b"IDAT", &idat_content);
}

/// Write central directory entries.
fn write_central_directory(buffer: &mut OutputBuffer, entries: &[FileEntry]) {
    for entry in entries {
        *buffer += b"PK\x01\x02";
        *buffer += &20_u16.to_le_bytes(); // version made by
        *buffer += &20_u16.to_le_bytes(); // version needed
        *buffer += &0_u16.to_le_bytes();  // flags
        *buffer += &8_u16.to_le_bytes();  // compression
        *buffer += &0_u16.to_le_bytes();  // mod time
        *buffer += &0_u16.to_le_bytes();  // mod date
        *buffer += &entry.crc.to_le_bytes();
        *buffer += &entry.compressed_size.to_le_bytes();
        *buffer += &(entry.body.len() as u32).to_le_bytes();
        *buffer += &(entry.name.len() as u16).to_le_bytes();
        *buffer += &0_u16.to_le_bytes();  // extra len
        *buffer += &0_u16.to_le_bytes();  // comment len
        *buffer += &0_u16.to_le_bytes();  // disk number
        *buffer += &0_u16.to_le_bytes();  // internal attrs
        *buffer += &0_u32.to_le_bytes();  // external attrs
        *buffer += &entry.header_offset.to_le_bytes();
        *buffer += entry.name.as_slice();
    }
}

/// Write End of Central Directory record.
fn write_eocd(buffer: &mut OutputBuffer, num_entries: u16, cd_size: u32, cd_offset: u32) {
    *buffer += b"PK\x05\x06";
    *buffer += &0_u16.to_le_bytes(); // disk number
    *buffer += &0_u16.to_le_bytes(); // disk with CD
    *buffer += &num_entries.to_le_bytes();
    *buffer += &num_entries.to_le_bytes();
    *buffer += &cd_size.to_le_bytes();
    *buffer += &cd_offset.to_le_bytes();
    *buffer += &0_u16.to_le_bytes(); // comment len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_calculation() {
        // Small content should get minimum width
        assert_eq!(calculate_row_width(100), MIN_ROW_WIDTH);

        // 10KB: sqrt(10000) = 100
        let w = calculate_row_width(10_000);
        assert_eq!(w, 100, "Expected 100, got {}", w);

        // 40KB: sqrt(40000) ≈ 200
        let w = calculate_row_width(40_000);
        assert_eq!(w, 200, "Expected 200, got {}", w);
    }

    #[test]
    fn test_polyglot_structure() {
        let files = vec![(b"test.txt".as_ref(), b"Hello, World!".as_ref())];
        let result = build_polyglot(&files, 0, BitDepth::EightBit, Lightness, None);

        // Check PNG signature
        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");
        // Check ZIP EOCD exists
        assert!(result.windows(4).any(|w| w == b"PK\x05\x06"));
    }

    #[test]
    fn test_boundary_detection() {
        // Test the crosses_idat_boundary function
        assert!(!crosses_idat_boundary(0, 1000));
        assert!(!crosses_idat_boundary(60000, 65000));
        assert!(crosses_idat_boundary(60000, 70000)); // Crosses 65535
        assert!(!crosses_idat_boundary(65535, 70000)); // Starts at boundary
        assert!(crosses_idat_boundary(130000, 132000)); // Crosses 131070
    }

    #[test]
    fn test_data_to_filtered_pos() {
        let row_width = 100;
        // First byte of data is at filtered position 1 (after filter byte)
        assert_eq!(data_to_filtered_pos(0, row_width), 1);
        // Last byte of first row
        assert_eq!(data_to_filtered_pos(99, row_width), 100);
        // First byte of second row (filtered position 101 + 1 = 102)
        assert_eq!(data_to_filtered_pos(100, row_width), 102);
        // Second byte of second row
        assert_eq!(data_to_filtered_pos(101, row_width), 103);
    }

    #[test]
    fn test_large_content_multiple_files() {
        // Create content that would exceed 65535 bytes when filtered
        // This tests that boundary padding works
        let large_body = vec![0xAB_u8; 30_000];
        let files = vec![
            (b"file1.bin".as_ref(), large_body.as_slice()),
            (b"file2.bin".as_ref(), large_body.as_slice()),
            (b"file3.bin".as_ref(), large_body.as_slice()),
        ];

        let result = build_polyglot(&files, 0, BitDepth::EightBit, Lightness, None);

        // Check PNG signature
        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");
        // Check all three files have local headers
        let pk_count = result.windows(4).filter(|w| *w == b"PK\x03\x04").count();
        assert_eq!(pk_count, 3, "Expected 3 local file headers");
        // Check EOCD exists
        assert!(result.windows(4).any(|w| w == b"PK\x05\x06"));
    }

    #[test]
    fn test_very_large_content() {
        // Create 5 files of 40KB each = 200KB total
        // This crosses multiple IDAT boundaries
        let files: Vec<_> = (0..5)
            .map(|i| {
                let name = format!("file{}.bin", i);
                let body: Vec<u8> = (0..40_000).map(|j| ((i * 17 + j * 7) % 256) as u8).collect();
                (name.into_bytes(), body)
            })
            .collect();

        let file_refs: Vec<(&[u8], &[u8])> = files
            .iter()
            .map(|(n, b)| (n.as_slice(), b.as_slice()))
            .collect();

        let result = build_polyglot(&file_refs, 0, BitDepth::EightBit, Lightness, None);

        // Should be over 200KB
        assert!(result.len() > 200_000, "Result should be >200KB, got {}", result.len());

        // Check PNG signature
        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");

        // Check all five files have local headers
        let pk_count = result.windows(4).filter(|w| *w == b"PK\x03\x04").count();
        assert_eq!(pk_count, 5, "Expected 5 local file headers, got {}", pk_count);

        // Check EOCD exists
        assert!(result.windows(4).any(|w| w == b"PK\x05\x06"));
    }
}
