//! Polyglot PNG+ZIP file creation.
//!
//! Creates files that are simultaneously valid PNGs and valid ZIPs.
//!
//! ## Structure
//!
//! ```text
//! [PNG signature]           \
//! [IHDR chunk]               | PNG structure
//! [IDAT chunk - pixel data]  | (displays ZIP content as image)
//! [IEND chunk]              /
//! [ZIP local header 1]      \
//! [File data 1]              |
//! [ZIP local header 2]       | ZIP content (after IEND)
//! [File data 2]              | No PNG filter bytes here!
//! ...                        |
//! [ZIP Central Directory]    |
//! [ZIP EOCD]                /
//! ```
//!
//! PNG readers see a valid image. ZIP readers scan from the end,
//! find EOCD, and extract files from after IEND.

use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::png::{BitDepth, ColorMode, write_png_chunk, write_png_header, write_png_footer};

/// Information about a file entry for building the central directory.
#[derive(Debug, Clone)]
struct FileEntry {
    name: Vec<u8>,
    body: Vec<u8>,
    crc: u32,
    /// Offset of the local file header in the final file.
    header_offset: u32,
}

/// Build a polyglot PNG+ZIP file.
///
/// The ZIP content is placed after IEND, avoiding any PNG filter byte issues.
/// The PNG displays the ZIP data as pixel values.
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    width: u32,
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8> {
    // Step 1: Build the complete ZIP local entries (headers + data)
    let zip_data = build_zip_entries(files);

    // Step 2: Calculate dimensions for displaying ZIP data as pixels
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let row_width = (width as usize * bits_per_pixel + 7) / 8;
    let row_width = row_width.max(64); // Minimum reasonable row width
    let height = (zip_data.len() + row_width - 1) / row_width;

    // Recalculate effective width
    let effective_width = (row_width * 8) / bits_per_pixel;

    // Pad ZIP data to fill complete rows
    let mut padded_zip = zip_data.clone();
    padded_zip.resize(height * row_width, 0);

    // Step 3: Build the PNG with ZIP data as pixels
    let mut output = Vec::new();

    // IHDR
    write_png_header(&mut output, effective_width as u32, height as u32, bit_depth, color_mode);

    // PLTE
    if let Some(p) = palette {
        write_png_chunk(&mut output, b"PLTE", p);
    }

    // IDAT - the ZIP data becomes the PNG pixels
    let filtered_data = add_png_filter_bytes(&padded_zip, row_width);
    write_idat_stored(&mut output, &filtered_data);

    // IEND
    write_png_footer(&mut output);

    // Step 4: Append ZIP content AFTER IEND (this is the real ZIP data)
    let zip_start = output.len();

    // Build file entries with correct offsets
    let file_entries = build_file_entries(files, zip_start);

    // Write local headers and file data
    for entry in &file_entries {
        write_local_header(&mut output, &entry.name, entry.body.len(), entry.crc);
        output.extend_from_slice(&entry.body);
    }

    // Step 5: Append central directory
    let central_dir_start = output.len();
    write_central_directory(&mut output, &file_entries);
    let central_dir_end = output.len();

    // Step 6: Append EOCD
    write_eocd(
        &mut output,
        file_entries.len() as u16,
        (central_dir_end - central_dir_start) as u32,
        central_dir_start as u32,
    );

    output
}

/// Build ZIP local entries (headers + data) as raw bytes.
fn build_zip_entries(files: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut data = Vec::new();
    for (name, body) in files {
        write_local_header(&mut data, name, body.len(), crc32(body));
        data.extend_from_slice(body);
    }
    data
}

/// Build file entries with calculated offsets.
fn build_file_entries(files: &[(&[u8], &[u8])], start_offset: usize) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let mut current_offset = start_offset;

    for (name, body) in files {
        entries.push(FileEntry {
            name: name.to_vec(),
            body: body.to_vec(),
            crc: crc32(body),
            header_offset: current_offset as u32,
        });
        current_offset += 30 + name.len() + body.len();
    }

    entries
}

/// Write a ZIP local file header.
fn write_local_header(data: &mut Vec<u8>, name: &[u8], size: usize, crc: u32) {
    data.extend_from_slice(b"PK\x03\x04");
    data.extend_from_slice(&10_u16.to_le_bytes()); // version
    data.extend_from_slice(&0_u16.to_le_bytes());  // flags
    data.extend_from_slice(&0_u16.to_le_bytes());  // compression (stored)
    data.extend_from_slice(&0_u16.to_le_bytes());  // mod time
    data.extend_from_slice(&0_u16.to_le_bytes());  // mod date
    data.extend_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&(size as u32).to_le_bytes()); // compressed size
    data.extend_from_slice(&(size as u32).to_le_bytes()); // uncompressed size
    data.extend_from_slice(&(name.len() as u16).to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());  // extra len
    data.extend_from_slice(name);
}

/// Add PNG filter bytes (0x00 = None filter) at the start of each row.
fn add_png_filter_bytes(data: &[u8], row_width: usize) -> Vec<u8> {
    let mut filtered = Vec::new();
    for chunk in data.chunks(row_width) {
        filtered.push(0x00);
        filtered.extend_from_slice(chunk);
    }
    filtered
}

/// Write IDAT chunk with stored (uncompressed) deflate.
fn write_idat_stored(buffer: &mut Vec<u8>, filtered_data: &[u8]) {
    let mut idat_content = Vec::new();

    // zlib header
    let cmf: u8 = 0x78;
    let mut flg: u8 = 0x01;
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
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        idat_content.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        idat_content.extend_from_slice(&(chunk.len() as u16).not().to_le_bytes());
        idat_content.extend_from_slice(chunk);
    }

    // Adler-32
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
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(entry.body.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // extra len (0 in central dir)
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // comment len
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // disk number
        buffer.extend_from_slice(&0_u16.to_le_bytes());  // internal attrs
        buffer.extend_from_slice(&0_u32.to_le_bytes());  // external attrs
        buffer.extend_from_slice(&entry.header_offset.to_le_bytes());
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
    fn test_polyglot_structure() {
        let files = vec![
            (b"test.txt".as_ref(), b"Hello!".as_ref()),
        ];

        let result = build_polyglot(
            &files,
            64,
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

        // Check for local header (should be after IEND)
        let pk_pos = result.windows(4)
            .position(|w| w == b"PK\x03\x04")
            .expect("Local header not found");
        assert!(pk_pos > 8); // After PNG signature
    }

    #[test]
    fn test_png_filter_bytes() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let filtered = add_png_filter_bytes(&data, 4);
        // Should be: [0, 1, 2, 3, 4, 0, 5, 6, 7, 8]
        assert_eq!(filtered, vec![0, 1, 2, 3, 4, 0, 5, 6, 7, 8]);
    }
}
