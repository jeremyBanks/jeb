//! Polyglot PNG+ZIP file creation.
//!
//! Creates files that are simultaneously valid PNGs and valid ZIPs.
//!
//! Structure:
//! ```text
//! [PNG signature]
//! [IHDR chunk]
//! [PLTE chunk - if indexed color]
//! [IDAT chunk containing ZIP local file entries as pixel data]
//! [IEND chunk]
//! [ZIP Central Directory]
//! [ZIP EOCD]
//! ```
//!
//! PNG readers see a valid image (displaying the ZIP data as pixels).
//! ZIP readers scan from the end, find EOCD, and extract files from within IDAT.

use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::png::{BitDepth, ColorMode, write_png_chunk, write_png_header, write_png_footer};

/// Information about a file entry for building the central directory.
#[derive(Debug, Clone)]
struct FileEntry {
    name: Vec<u8>,
    body: Vec<u8>,
    crc: u32,
    /// Offset of the local file header in the FILTERED data (final file position).
    filtered_offset: u32,
}

/// Minimum row width in bytes for polyglot files.
/// Must be large enough to contain a ZIP local header (30 bytes) plus filename plus file data.
/// Filter bytes are inserted at row boundaries, so file data must fit within a single row
/// to avoid corruption. Using 4096 bytes allows files up to ~4KB per entry.
/// For larger files, the row width should be increased accordingly.
const MIN_POLYGLOT_ROW_WIDTH: usize = 4096;

/// Build a polyglot PNG+ZIP file with proper 2D layout.
///
/// Entries are aligned to row boundaries so that PNG filter bytes
/// land in padding between entries, not inside ZIP structures.
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    width: u32,
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8> {
    // Calculate bytes per row correctly for sub-byte bit depths
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let mut row_width = (width as usize * bits_per_pixel + 7) / 8;

    // Calculate minimum row width needed for the largest entry
    // Each entry needs: header(30) + name + extra_padding(up to row_width) + body
    // The body must fit in remaining row space after the header reaches a boundary
    let max_body_size = files.iter().map(|(_, b)| b.len()).max().unwrap_or(0);
    let needed_row_width = MIN_POLYGLOT_ROW_WIDTH.max(max_body_size + 64); // +64 for header overhead

    // For polyglot files, enforce minimum row width to fit entries
    // If row_width is too small, increase it and recalculate effective width
    let effective_width = if row_width < needed_row_width {
        row_width = needed_row_width;
        // Calculate pixels that fit in this row width
        (row_width * 8) / bits_per_pixel
    } else {
        width as usize
    };

    // Step 1: Build row-aligned ZIP local entries
    let (local_data, entry_infos) = build_row_aligned_entries(files, row_width);

    // Step 2: Calculate PNG dimensions
    let height = (local_data.len() + row_width - 1) / row_width;

    // Ensure data fills complete rows
    let mut padded_data = local_data;
    let padded_len = height * row_width;
    padded_data.resize(padded_len, 0);

    // Step 3: Calculate PNG prefix size for offset calculation
    let png_sig_size = 8;
    let ihdr_size = 4 + 4 + 13 + 4; // 25 bytes
    let plte_size = palette.map(|p| 4 + 4 + p.len() + 4).unwrap_or(0);
    let idat_header_size = 4 + 4; // chunk length + "IDAT"
    let zlib_header_size = 2;
    let deflate_header_size = 5; // for first stored block

    let data_start_in_file = png_sig_size
        + ihdr_size
        + plte_size
        + idat_header_size
        + zlib_header_size
        + deflate_header_size;

    // Step 4: Convert original positions to filtered (final file) positions
    let file_entries: Vec<FileEntry> = entry_infos
        .into_iter()
        .map(|(name, body, original_pos)| {
            let filtered_pos = original_to_filtered_pos(original_pos, row_width);
            let final_offset = data_start_in_file + filtered_pos;
            FileEntry {
                name: name.to_vec(),
                body: body.to_vec(),
                crc: crc32(body),
                filtered_offset: final_offset as u32,
            }
        })
        .collect();

    // Step 5: Build the PNG
    let mut output = Vec::new();

    // IHDR (includes PNG signature)
    // Use effective_width to match the actual row structure
    write_png_header(&mut output, effective_width as u32, height as u32, bit_depth, color_mode);

    // PLTE (if indexed)
    if let Some(p) = palette {
        write_png_chunk(&mut output, b"PLTE", p);
    }

    // IDAT with filtered data
    let filtered_data = add_png_filter_bytes(&padded_data, row_width);
    write_idat_stored(&mut output, &filtered_data);

    // IEND
    write_png_footer(&mut output);

    // Step 6: Append ZIP central directory
    let central_dir_start = output.len();
    write_central_directory(&mut output, &file_entries);
    let central_dir_end = output.len();

    // Step 7: Append ZIP EOCD
    write_eocd(
        &mut output,
        file_entries.len() as u16,
        (central_dir_end - central_dir_start) as u32,
        central_dir_start as u32,
    );

    output
}

/// Build ZIP local entries with careful filter byte alignment.
///
/// Strategy:
/// 1. Entry header starts at a row boundary (filter byte before PK signature)
/// 2. File data starts at a row boundary (extra field pads header to boundary)
/// 3. Extra field length includes the filter byte count to help ZIP navigate
/// 4. File data must fit within row_width to avoid internal filter bytes
///
/// This ensures PNG filter bytes land in predictable locations that don't
/// corrupt the ZIP structure or file contents.
fn build_row_aligned_entries<'a>(
    files: &[(&'a [u8], &'a [u8])],
    row_width: usize,
) -> (Vec<u8>, Vec<(&'a [u8], &'a [u8], usize)>) {
    let mut data = Vec::new();
    let mut entry_infos = Vec::new();

    // Fixed header size (before filename)
    const LOCAL_HEADER_FIXED: usize = 30;

    for (name, body) in files {
        // Pad to align entry start to row boundary
        if !data.is_empty() {
            let current_pos = data.len();
            let padding_needed = (row_width - (current_pos % row_width)) % row_width;
            data.resize(data.len() + padding_needed, 0);
        }

        let entry_start = data.len();

        // Calculate header size (fixed + name)
        let header_size = LOCAL_HEADER_FIXED + name.len();

        // Calculate extra field size to push file data to next row boundary
        // This padding will include the filter byte when ZIP navigates
        let header_end_in_row = header_size % row_width;
        let extra_content_len = if header_end_in_row == 0 {
            0 // Header already ends at boundary
        } else {
            row_width - header_end_in_row
        };

        // The extra_len in the ZIP header must include +1 for each filter byte
        // that ZIP will encounter when skipping past the extra field.
        // If header+name+extra_content spans exactly to a row boundary,
        // there's 1 filter byte between extra and file data.
        let filter_bytes_before_data = if extra_content_len > 0 { 1 } else { 0 };
        let extra_len_for_zip = extra_content_len + filter_bytes_before_data;

        entry_infos.push((*name, *body, entry_start));

        // Build local file header
        data.extend_from_slice(b"PK\x03\x04");
        data.extend_from_slice(&10_u16.to_le_bytes()); // version needed
        data.extend_from_slice(&0_u16.to_le_bytes());  // flags
        data.extend_from_slice(&0_u16.to_le_bytes());  // compression (stored)
        data.extend_from_slice(&0_u16.to_le_bytes());  // mod time
        data.extend_from_slice(&0_u16.to_le_bytes());  // mod date
        data.extend_from_slice(&crc32(body).to_le_bytes());
        data.extend_from_slice(&(body.len() as u32).to_le_bytes()); // compressed size
        data.extend_from_slice(&(body.len() as u32).to_le_bytes()); // uncompressed size
        data.extend_from_slice(&(name.len() as u16).to_le_bytes());
        data.extend_from_slice(&(extra_len_for_zip as u16).to_le_bytes());

        // Filename
        data.extend_from_slice(name);

        // Extra field content (actual padding bytes, not including filter byte)
        data.resize(data.len() + extra_content_len, 0);

        // File data (now at row boundary in original data)
        data.extend_from_slice(body);
    }

    (data, entry_infos)
}

/// Convert original data position to filtered position.
///
/// PNG filtering inserts a filter byte at the start of each row.
/// For row width W, original position P maps to filtered position:
///   F = P + (P / W) + 1
/// (one filter byte per complete row, plus one for the current row)
fn original_to_filtered_pos(original_pos: usize, row_width: usize) -> usize {
    let row = original_pos / row_width;
    original_pos + row + 1
}

/// Add PNG filter bytes (0x00 = None filter) at the start of each row.
fn add_png_filter_bytes(data: &[u8], row_width: usize) -> Vec<u8> {
    let mut filtered = Vec::new();

    for chunk in data.chunks(row_width) {
        filtered.push(0x00); // Filter type: None
        filtered.extend_from_slice(chunk);
    }

    filtered
}

/// Write IDAT chunk with stored (uncompressed) deflate.
fn write_idat_stored(buffer: &mut Vec<u8>, filtered_data: &[u8]) {
    let mut idat_content = Vec::new();

    // zlib header: CMF=0x78 (deflate, 32K window), FLG for checksum
    let cmf: u8 = 0x78;
    let mut flg: u8 = 0x01;
    let check = ((cmf as u16) * 256 + (flg as u16)) % 31;
    if check != 0 {
        flg += (31 - check) as u8;
    }
    idat_content.push(cmf);
    idat_content.push(flg);

    // Deflate stored blocks (max 65535 bytes each)
    let chunks: Vec<&[u8]> = filtered_data.chunks(65535).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        idat_content.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        idat_content.extend_from_slice(&(chunk.len() as u16).not().to_le_bytes());
        idat_content.extend_from_slice(chunk);
    }

    // Adler-32 checksum (big-endian for zlib)
    idat_content.extend_from_slice(&adler32(filtered_data).to_be_bytes());

    write_png_chunk(buffer, b"IDAT", &idat_content);
}

/// Write ZIP central directory entries.
fn write_central_directory(buffer: &mut Vec<u8>, entries: &[FileEntry]) {
    for entry in entries {
        buffer.extend_from_slice(b"PK\x01\x02");
        buffer.extend_from_slice(&20_u16.to_le_bytes()); // version made by
        buffer.extend_from_slice(&10_u16.to_le_bytes()); // version needed
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // flags
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // compression
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // mod time
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // mod date
        buffer.extend_from_slice(&entry.crc.to_le_bytes());
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes()); // compressed
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes()); // uncompressed
        buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // extra len
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // comment len
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // disk number
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // internal attrs
        buffer.extend_from_slice(&0_u32.to_le_bytes());  // external attrs
        buffer.extend_from_slice(&entry.filtered_offset.to_le_bytes()); // local header offset
        buffer.extend_from_slice(&entry.name);
    }
}

/// Write ZIP end of central directory record.
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
    fn test_row_aligned_polyglot() {
        let files = vec![
            (b"hello.txt".as_ref(), b"Hello, World!".as_ref()),
            (b"test.txt".as_ref(), b"Test content here".as_ref()),
        ];

        let result = build_polyglot(
            &files,
            64, // 64 pixels wide
            BitDepth::EightBit,
            ColorMode::Lightness,
            None,
        );

        // Check PNG signature
        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");

        // Check for ZIP EOCD
        let eocd_pos = result.windows(4)
            .rposition(|w| w == b"PK\x05\x06")
            .expect("EOCD not found");
        assert!(eocd_pos > 0);

        println!("Polyglot size: {} bytes", result.len());
        println!("EOCD at offset: {}", eocd_pos);
    }

    #[test]
    fn test_original_to_filtered() {
        // With row_width = 64:
        // Position 0 -> filtered 1 (after first filter byte)
        // Position 64 -> filtered 66 (row 1, after 2 filter bytes)
        // Position 128 -> filtered 131 (row 2, after 3 filter bytes)
        assert_eq!(original_to_filtered_pos(0, 64), 1);
        assert_eq!(original_to_filtered_pos(63, 64), 64);
        assert_eq!(original_to_filtered_pos(64, 64), 66);
        assert_eq!(original_to_filtered_pos(128, 64), 131);
    }
}
