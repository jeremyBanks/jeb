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
    /// Offset of the local file header from the start of the final file.
    header_offset: u32,
}

/// Build a polyglot PNG+ZIP file.
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    width: u32,
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8> {
    // Step 1: Build the ZIP local file entries (headers + data)
    let local_entries = build_local_entries(files);

    // Step 2: Calculate PNG parameters
    // For polyglot to work, we need to minimize filter byte interference.
    // Using the entire data as one row means only ONE filter byte at the start,
    // keeping all ZIP data contiguous.
    let bytes_per_pixel = (bit_depth.bits_per_sample() * color_mode.samples_per_pixel() + 7) / 8;
    let bytes_per_pixel = bytes_per_pixel.max(1);

    // Calculate the actual width to use - prefer single row for ZIP integrity
    let data_len = local_entries.len();
    let (actual_width, height) = if data_len <= 65535 {
        // Single row - all data contiguous after one filter byte
        let actual_width = (data_len + bytes_per_pixel - 1) / bytes_per_pixel;
        (actual_width, 1)
    } else {
        // Multiple rows needed - use requested width
        let bytes_per_row = width as usize * bytes_per_pixel;
        let height = (data_len + bytes_per_row - 1) / bytes_per_row;
        (width as usize, height)
    };

    let bytes_per_row = actual_width * bytes_per_pixel;

    // Pad data to fill complete rows
    let padded_len = height * bytes_per_row;
    let mut padded_data = local_entries.clone();
    padded_data.resize(padded_len, 0);

    // Use actual_width for PNG
    let width = actual_width as u32;

    // Step 3: Calculate the PNG prefix size (everything before IDAT data)
    let png_sig_size = 8;
    let ihdr_size = 4 + 4 + 13 + 4; // len + type + data + crc = 25
    let plte_size = if let Some(p) = palette {
        4 + 4 + p.len() + 4 // len + type + data + crc
    } else {
        0
    };
    let idat_header_size = 4 + 4; // len + type
    let zlib_header_size = 2;

    // Calculate deflate overhead (stored blocks have 5-byte headers every 65535 bytes)
    let filtered_data_len = padded_len + height; // +1 filter byte per row
    let num_deflate_blocks = (filtered_data_len + 65534) / 65535;
    let deflate_headers_size = num_deflate_blocks * 5;

    // The first byte of actual data starts at this offset
    let data_start_offset = png_sig_size
        + ihdr_size
        + plte_size
        + idat_header_size
        + zlib_header_size
        + 5; // first deflate block header

    // Step 4: Calculate where each local file header ends up in the final file
    let file_entries = calculate_file_offsets(
        files,
        &local_entries,
        data_start_offset,
        bytes_per_row,
    );

    // Step 5: Build the PNG
    let mut output = Vec::new();

    // IHDR (write_png_header includes the PNG signature)
    write_png_header(&mut output, width, height as u32, bit_depth, color_mode);

    // PLTE (if indexed)
    if let Some(p) = palette {
        write_png_chunk(&mut output, b"PLTE", p);
    }

    // IDAT with filtered data
    let filtered_data = add_png_filter_bytes(&padded_data, bytes_per_row);
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

/// Build ZIP local file entries (header + data for each file).
fn build_local_entries(files: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut data = Vec::new();

    for (name, body) in files {
        // Local file header
        // 0x0000..0x0004: signature
        data.extend_from_slice(b"PK\x03\x04");
        // 0x0004..0x0006: version needed (1.0)
        data.extend_from_slice(&10_u16.to_le_bytes());
        // 0x0006..0x0008: general purpose bit flag
        data.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0008..0x000A: compression method (0 = stored)
        data.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000A..0x000C: last mod time
        data.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000C..0x000E: last mod date
        data.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000E..0x0012: CRC-32
        data.extend_from_slice(&crc32(body).to_le_bytes());
        // 0x0012..0x0016: compressed size
        data.extend_from_slice(&(body.len() as u32).to_le_bytes());
        // 0x0016..0x001A: uncompressed size
        data.extend_from_slice(&(body.len() as u32).to_le_bytes());
        // 0x001A..0x001C: file name length
        data.extend_from_slice(&(name.len() as u16).to_le_bytes());
        // 0x001C..0x001E: extra field length
        data.extend_from_slice(&0_u16.to_le_bytes());
        // File name
        data.extend_from_slice(name);
        // File data (no extra field)
        data.extend_from_slice(body);
    }

    data
}

/// Calculate the actual file offset for each local file header after PNG encoding.
fn calculate_file_offsets(
    files: &[(&[u8], &[u8])],
    local_entries: &[u8],
    data_start_offset: usize,
    bytes_per_row: usize,
) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let mut pos = 0; // Position in local_entries

    for (name, body) in files {
        // Calculate the filtered position (accounting for PNG filter bytes)
        let row = pos / bytes_per_row;
        let col = pos % bytes_per_row;
        let filtered_pos = row * (bytes_per_row + 1) + 1 + col; // +1 for filter byte at start of each row

        // Calculate deflate block position
        let deflate_block = filtered_pos / 65535;
        let pos_in_block = filtered_pos % 65535;
        let deflate_pos = deflate_block * (65535 + 5) + pos_in_block;

        // Final file offset
        let file_offset = data_start_offset + deflate_pos;

        entries.push(FileEntry {
            name: name.to_vec(),
            body: body.to_vec(),
            crc: crc32(body),
            header_offset: file_offset as u32,
        });

        // Advance position past this entry
        let header_size = 30 + name.len();
        pos += header_size + body.len();
    }

    entries
}

/// Add PNG filter bytes (0x00 = None filter) at the start of each row.
fn add_png_filter_bytes(data: &[u8], bytes_per_row: usize) -> Vec<u8> {
    let mut filtered = Vec::new();

    for chunk in data.chunks(bytes_per_row) {
        filtered.push(0x00); // Filter type: None
        filtered.extend_from_slice(chunk);
    }

    filtered
}

/// Write IDAT chunk with stored (uncompressed) deflate.
fn write_idat_stored(buffer: &mut Vec<u8>, filtered_data: &[u8]) {
    let mut idat_content = Vec::new();

    // zlib header: CMF=0x78 (deflate, 32K window), FLG calculated for checksum
    let cmf: u8 = 0x78;
    let mut flg: u8 = 0x01; // compression level 0
    // Adjust FLG so (CMF * 256 + FLG) % 31 == 0
    let check = ((cmf as u16) * 256 + (flg as u16)) % 31;
    if check != 0 {
        flg += (31 - check) as u8;
    }
    idat_content.push(cmf);
    idat_content.push(flg);

    // Deflate stored blocks
    let chunks: Vec<&[u8]> = filtered_data.chunks(65535).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        // BFINAL (1 bit) + BTYPE=00 (2 bits) = stored block
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        // LEN (16-bit little-endian)
        idat_content.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        // NLEN (one's complement of LEN)
        idat_content.extend_from_slice(&(chunk.len() as u16).not().to_le_bytes());
        // Data
        idat_content.extend_from_slice(chunk);
    }

    // Adler-32 checksum of uncompressed data (big-endian for zlib!)
    idat_content.extend_from_slice(&adler32(filtered_data).to_be_bytes());

    // Write as PNG chunk
    write_png_chunk(buffer, b"IDAT", &idat_content);
}

/// Write ZIP central directory entries.
fn write_central_directory(buffer: &mut Vec<u8>, entries: &[FileEntry]) {
    for entry in entries {
        // Central directory file header
        // 0x0000..0x0004: signature
        buffer.extend_from_slice(b"PK\x01\x02");
        // 0x0004..0x0006: version made by
        buffer.extend_from_slice(&20_u16.to_le_bytes());
        // 0x0006..0x0008: version needed
        buffer.extend_from_slice(&10_u16.to_le_bytes());
        // 0x0008..0x000A: general purpose bit flag
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000A..0x000C: compression method (0 = stored)
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000C..0x000E: last mod time
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x000E..0x0010: last mod date
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0010..0x0014: CRC-32
        buffer.extend_from_slice(&entry.crc.to_le_bytes());
        // 0x0014..0x0018: compressed size
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes());
        // 0x0018..0x001C: uncompressed size
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes());
        // 0x001C..0x001E: file name length
        buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
        // 0x001E..0x0020: extra field length
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0020..0x0022: file comment length
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0022..0x0024: disk number start
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0024..0x0026: internal file attributes
        buffer.extend_from_slice(&0_u16.to_le_bytes());
        // 0x0026..0x002A: external file attributes
        buffer.extend_from_slice(&0_u32.to_le_bytes());
        // 0x002A..0x002E: relative offset of local header
        buffer.extend_from_slice(&entry.header_offset.to_le_bytes());
        // File name
        buffer.extend_from_slice(&entry.name);
    }
}

/// Write ZIP end of central directory record.
fn write_eocd(buffer: &mut Vec<u8>, num_entries: u16, central_dir_size: u32, central_dir_offset: u32) {
    // 0x0000..0x0004: signature
    buffer.extend_from_slice(b"PK\x05\x06");
    // 0x0004..0x0006: disk number
    buffer.extend_from_slice(&0_u16.to_le_bytes());
    // 0x0006..0x0008: disk number with central directory
    buffer.extend_from_slice(&0_u16.to_le_bytes());
    // 0x0008..0x000A: number of entries on this disk
    buffer.extend_from_slice(&num_entries.to_le_bytes());
    // 0x000A..0x000C: total number of entries
    buffer.extend_from_slice(&num_entries.to_le_bytes());
    // 0x000C..0x0010: size of central directory
    buffer.extend_from_slice(&central_dir_size.to_le_bytes());
    // 0x0010..0x0014: offset of central directory
    buffer.extend_from_slice(&central_dir_offset.to_le_bytes());
    // 0x0014..0x0016: comment length
    buffer.extend_from_slice(&0_u16.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_polyglot_basic() {
        let files = vec![
            (b"hello.txt".as_ref(), b"Hello, World!".as_ref()),
        ];

        let result = build_polyglot(
            &files,
            64, // width
            BitDepth::EightBit,
            ColorMode::Lightness,
            None,
        );

        // Check PNG signature
        assert_eq!(&result[0..8], b"\x89PNG\r\n\x1A\n");

        // Check for ZIP EOCD signature near the end
        let eocd_pos = result.windows(4)
            .rposition(|w| w == b"PK\x05\x06")
            .expect("EOCD not found");
        assert!(eocd_pos > 0);

        println!("Polyglot size: {} bytes", result.len());
        println!("EOCD at offset: {}", eocd_pos);
    }
}
