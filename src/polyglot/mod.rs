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
//! - Minimum width of 64 ensures filter bytes don't corrupt ZIP headers
//! - Height >= width is preferred (portrait/square orientation)
//!
//! ## IDAT Block Boundaries
//!
//! IDAT uses stored deflate blocks with max 65535 bytes each. Files are
//! automatically padded to avoid crossing block boundaries, allowing
//! unlimited total content (individual files limited to ~60KB each).
//!
//! ## Filename Labels
//!
//! Filename labels are rendered above each file's content using size-appropriate
//! bitmap fonts:
//! - ≤ 128 KiB: Sky (9×10 pixels)
//! - ≤ 512 KiB: Sugimori (8×8 pixels)
//! - ≤ 1 MiB: Mini (3×6 pixels)
//! - ≤ 3 MiB: Micro (3×3 pixels)
//! - > 3 MiB: No labels

mod fonts;

#[cfg(test)]
mod validate;
#[cfg(test)]
pub use validate::{validate_polyglot, assert_valid_polyglot, ValidationResult};

use std::collections::HashSet;
use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::io::{OutputBuffer, output_buffer};
use crate::png::write_png::{write_png_header, write_png_chunk, write_png_footer};
use fonts::{BitmapFont, select_font};

// Re-export types from png module with compatibility aliases
pub use crate::png::{BitDepth, ColorType};
pub use crate::png::BitDepth::*;
pub use crate::png::ColorType::*;

/// Alias for backward compatibility - ColorMode is now ColorType
pub type ColorMode = ColorType;

/// Alias for backward compatibility - Lightness is now Luminance
pub const Lightness: ColorType = ColorType::Luminance;
pub const LightnessAlpha: ColorType = ColorType::LuminanceAlpha;

/// Row width alignment - all row widths are multiples of this value.
/// This ensures consistent data alignment for better compression and structure.
const ROW_WIDTH_ALIGNMENT: usize = 64;

/// Base minimum row width (must be a multiple of ROW_WIDTH_ALIGNMENT).
const BASE_MIN_ROW_WIDTH: usize = 64;

/// Maximum bytes per IDAT deflate stored block.
const IDAT_BLOCK_SIZE: usize = 65535;

/// Maximum size for a single file's compressed content.
/// Must fit within one IDAT block (65535 bytes of filtered data).
pub const MAX_FILE_CONTENT_SIZE: usize = 60_000;

/// Maximum image height before disabling filename labels.
const MAX_HEIGHT_WITH_LABELS: usize = 1024;

/// Check if two glyphs would touch at a given horizontal offset.
/// Returns true if any pixels are 8-directionally adjacent.
fn glyphs_touch(prev: &[Vec<bool>], next: &[Vec<bool>], offset: i32) -> bool {
    let prev_width = prev.first().map(|r| r.len()).unwrap_or(0);
    let next_width = next.first().map(|r| r.len()).unwrap_or(0);

    for (prev_row, prev_pixels) in prev.iter().enumerate() {
        for (prev_col, &prev_on) in prev_pixels.iter().enumerate() {
            if !prev_on {
                continue;
            }
            // Check if any pixel in next glyph is adjacent
            for (next_row, next_pixels) in next.iter().enumerate() {
                for (next_col, &next_on) in next_pixels.iter().enumerate() {
                    if !next_on {
                        continue;
                    }
                    let dx = (next_col as i32 + offset) - prev_col as i32;
                    let dy = next_row as i32 - prev_row as i32;
                    if dx.abs() <= 1 && dy.abs() <= 1 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Calculate minimum gap between two glyphs (ensuring no 8-directional adjacency).
/// Returns a NEGATIVE value if glyphs can overlap, positive if they need extra space.
fn min_glyph_gap(prev: &[Vec<bool>], next: &[Vec<bool>], glyph_width: usize) -> i32 {
    // Try negative gaps first (tighter kerning)
    for gap in (-(glyph_width as i32 - 1))..((glyph_width * 2) as i32) {
        let offset = glyph_width as i32 + gap;
        if offset > 0 && !glyphs_touch(prev, next, offset) {
            return gap;
        }
    }
    glyph_width as i32 // fallback: full glyph width gap
}

/// Calculate the number of label rows for a given font.
fn label_rows_for_font(font: &BitmapFont) -> usize {
    // 1 empty row above + font height + 1 empty row below
    1 + font.height + 1
}

/// Render filename label rows using the specified font.
/// The header_row bytes are "stretched" into each label row as a background,
/// then text is rendered on top with a 1-pixel halo erased for readability.
/// Kerning checks against the accumulated rendering to avoid touching earlier chars.
fn render_filename_label(name: &[u8], row_width: usize, font: &BitmapFont, header_row: &[u8]) -> Vec<u8> {
    let label_rows = label_rows_for_font(font);
    let mut result = Vec::with_capacity(label_rows * row_width);

    // Convert name to chars and get glyphs
    let chars: Vec<char> = name.iter().map(|&b| b as char).collect();
    let glyphs: Vec<Option<&Vec<Vec<bool>>>> = chars.iter()
        .map(|&c| font.get_glyph(c))
        .collect();

    // Build accumulated canvas for kerning (checks against ALL previous chars, not just one)
    // Canvas is font.height rows × max_possible_width columns
    let max_canvas_width = row_width * 2; // generous size
    let mut canvas: Vec<Vec<bool>> = vec![vec![false; max_canvas_width]; font.height];
    let mut char_positions: Vec<(usize, i32)> = Vec::new();
    let mut total_width = 0i32;
    let mut rightmost_pixel = 0i32;

    for (i, glyph_opt) in glyphs.iter().enumerate() {
        if let Some(glyph) = glyph_opt {
            // Find minimum position where this glyph doesn't touch the canvas
            let mut best_offset = total_width + font.width as i32; // default: after previous char

            // Try tighter positions (can overlap into previous char's bounding box)
            for test_offset in (total_width - font.width as i32 + 1)..=best_offset {
                if test_offset < 0 {
                    continue;
                }
                let mut touches = false;
                'check: for (gy, glyph_row) in glyph.iter().enumerate() {
                    for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                        if !pixel_on {
                            continue;
                        }
                        let cx = test_offset as usize + gx;
                        // Check 8-directional adjacency against canvas
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                let ny = gy as i32 + dy;
                                let nx = cx as i32 + dx;
                                if ny >= 0 && (ny as usize) < font.height && nx >= 0 {
                                    if canvas[ny as usize].get(nx as usize).copied().unwrap_or(false) {
                                        touches = true;
                                        break 'check;
                                    }
                                }
                            }
                        }
                    }
                }
                if !touches {
                    best_offset = test_offset;
                    break;
                }
            }

            char_positions.push((i, best_offset));

            // Add glyph to canvas at best_offset
            for (gy, glyph_row) in glyph.iter().enumerate() {
                for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                    if pixel_on {
                        let cx = best_offset as usize + gx;
                        if cx < max_canvas_width {
                            canvas[gy][cx] = true;
                        }
                        rightmost_pixel = rightmost_pixel.max(cx as i32 + 1);
                    }
                }
            }

            total_width = best_offset + font.width as i32;
        }
    }

    // Use actual rendered width (excluding trailing blank pixels)
    let actual_width = rightmost_pixel.max(0) as usize;

    // Calculate starting x position (right-align if too long)
    // No margin needed since entire row is header background
    let start_x: i32 = if actual_width <= row_width {
        0 // Left-aligned at edge
    } else {
        // Right-aligned: truncate from left
        (row_width as i32 - actual_width as i32).max(-(actual_width as i32))
    };

    // First, render text to a temporary bitmap to know where pixels are
    let mut text_bitmap: Vec<Vec<bool>> = vec![vec![false; row_width]; label_rows];

    // Padding row above (row 0) has no text
    // Text rows are 1..=font.height
    // Padding row below is font.height + 1

    for text_row in 0..font.height {
        let result_row = text_row + 1; // +1 for padding above
        for &(char_idx, char_x) in &char_positions {
            if let Some(glyph) = glyphs[char_idx] {
                if text_row < glyph.len() {
                    for (px, &pixel_on) in glyph[text_row].iter().enumerate() {
                        let x = (start_x + char_x + px as i32) as usize;
                        if x < row_width && pixel_on {
                            text_bitmap[result_row][x] = true;
                        }
                    }
                }
            }
        }
    }

    // Now render each label row: header background, then erase halo, then draw text
    for row_idx in 0..label_rows {
        let mut row = vec![0u8; row_width];

        // Copy header row as background (stretched)
        let copy_len = row_width.min(header_row.len());
        row[..copy_len].copy_from_slice(&header_row[..copy_len]);

        // Erase pixels near text: 2 pixels horizontally, 1 pixel vertically
        for x in 0..row_width {
            let mut near_text = false;
            'halo: for dy in -1i32..=1 {
                for dx in -2i32..=2 {
                    let ny = row_idx as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny >= 0 && (ny as usize) < label_rows && nx >= 0 && (nx as usize) < row_width {
                        if text_bitmap[ny as usize][nx as usize] {
                            near_text = true;
                            break 'halo;
                        }
                    }
                }
            }
            if near_text {
                row[x] = 0x00; // erase to black (creates readable halo around text)
            }
        }

        // Draw text pixels on top (white)
        for x in 0..row_width {
            if text_bitmap[row_idx][x] {
                row[x] = 0xFF;
            }
        }

        result.extend_from_slice(&row);
    }

    result
}

/// Estimate total data size for width calculation.
fn estimate_total_size(files: &[(&[u8], &[u8])], font: Option<&BitmapFont>) -> usize {
    let mut total = 0;
    for (name, body) in files {
        // ZIP header: 30 bytes + name length + padding estimate
        total += 30 + name.len() + 50;
        // File content + deflate overhead (~4 bytes per 36 bytes)
        total += body.len();
        total += (body.len() / 36 + 1) * 4;
    }
    // Add label overhead estimate based on font
    if let Some(f) = font {
        let estimated_width = (total.max(100) as f64).sqrt() as usize;
        let label_rows = label_rows_for_font(f);
        total += files.len() * label_rows * estimated_width.max(BASE_MIN_ROW_WIDTH);
    }
    // Minimum reasonable size
    total.max(100)
}

/// Round up to the nearest multiple of ROW_WIDTH_ALIGNMENT.
fn align_to_row_width(value: usize) -> usize {
    ((value + ROW_WIDTH_ALIGNMENT - 1) / ROW_WIDTH_ALIGNMENT) * ROW_WIDTH_ALIGNMENT
}

/// Calculate optimal row width for approximately square images.
/// Returns width such that height >= width (portrait/square orientation).
/// Width is always a multiple of ROW_WIDTH_ALIGNMENT (64 bytes).
///
/// The `min_width` parameter ensures the row is wide enough to fit ZIP headers
/// without spanning multiple rows (which would corrupt them with filter bytes).
fn calculate_row_width(total_data_estimate: usize, min_width: usize) -> usize {
    // For height >= width, we need: total_data / width >= width
    // Therefore: width <= sqrt(total_data)
    // Using floor ensures height >= width
    let ideal = (total_data_estimate as f64).sqrt();
    let width = ideal.floor() as usize;

    // Clamp to minimum safe width and align to ROW_WIDTH_ALIGNMENT
    align_to_row_width(width.max(min_width))
}

/// Calculate minimum row width needed for a set of files.
/// Ensures ZIP local file headers (30 bytes + filename) fit in one row.
/// Returns a value aligned to ROW_WIDTH_ALIGNMENT.
fn min_row_width_for_files(files: &[(&[u8], &[u8])]) -> usize {
    let longest_filename = files.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    // Header is 30 bytes + filename; add 4 bytes padding for safety
    let min_for_headers = 30 + longest_filename + 4;
    align_to_row_width(min_for_headers.max(BASE_MIN_ROW_WIDTH))
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
///
/// # Panics
/// Panics if any file exceeds MAX_FILE_CONTENT_SIZE (~60KB). Large files cannot
/// be supported because they would span IDAT block boundaries, corrupting the
/// deflate stream.
pub fn build_polyglot(
    files: &[(&[u8], &[u8])],
    _width: u32,
    bit_depth: BitDepth,
    color_mode: ColorMode,
    palette: Option<&[u8]>,
) -> Vec<u8> {
    // Validate file sizes - files larger than MAX_FILE_CONTENT_SIZE will span
    // IDAT boundaries and produce corrupt deflate streams
    for (name, body) in files {
        if body.len() > MAX_FILE_CONTENT_SIZE {
            let name_str = String::from_utf8_lossy(name);
            panic!(
                "File '{}' is {} bytes, exceeding maximum of {} bytes. \
                 Large files cannot be embedded in polyglot PNG+ZIP because they \
                 would span IDAT block boundaries.",
                name_str, body.len(), MAX_FILE_CONTENT_SIZE
            );
        }
    }

    // Calculate minimum row width based on filename lengths
    let min_width = min_row_width_for_files(files);

    // Calculate total content size to select appropriate font
    let total_content_size: usize = files.iter().map(|(_, body)| body.len()).sum();

    // Select font based on content size (None if too large for labels)
    let font = select_font(total_content_size);

    // Two-pass approach for optimal dimensions:
    // Pass 1: Build with estimated width to get actual data size
    // Pass 2: Rebuild with width calculated from actual size

    // Pass 1: Use estimate for initial build
    let estimated_size = estimate_total_size(files, font);
    let initial_width = calculate_row_width(estimated_size, min_width);
    let (initial_data, _, _) = build_aligned_data(files, initial_width, font);

    // Pass 2: Calculate optimal width from actual size, rebuild
    let actual_size = initial_data.len();
    let row_width = calculate_row_width(actual_size, min_width);
    let (pixel_data, entry_infos, final_block_rows) = build_aligned_data(files, row_width, font);

    // Check if height exceeds limit and retry without labels if needed
    let height = if pixel_data.is_empty() { 1 } else { (pixel_data.len() + row_width - 1) / row_width };
    let (pixel_data, entry_infos, final_block_rows, row_width) = if font.is_some() && height > MAX_HEIGHT_WITH_LABELS {
        // Rebuild without labels
        let estimated_size = estimate_total_size(files, None);
        let initial_width = calculate_row_width(estimated_size, min_width);
        let (initial_data, _, _) = build_aligned_data(files, initial_width, None);
        let actual_size = initial_data.len();
        let row_width = calculate_row_width(actual_size, min_width);
        let (pixel_data, entry_infos, final_block_rows) = build_aligned_data(files, row_width, None);
        (pixel_data, entry_infos, final_block_rows, row_width)
    } else {
        (pixel_data, entry_infos, final_block_rows, row_width)
    };

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

/// Calculate the total size a file will occupy (label + header + content).
fn calculate_file_size(name: &[u8], body: &[u8], row_width: usize, font: Option<&BitmapFont>) -> usize {
    let data_per_block = row_width - 4;
    let header_size = 30 + name.len();
    let bytes_for_alignment = header_size % row_width;
    let extra_len = if bytes_for_alignment == 0 { 0 } else { row_width - bytes_for_alignment };
    let num_blocks = if body.is_empty() { 1 } else { (body.len() + data_per_block - 1) / data_per_block };
    let file_data_size = header_size + extra_len + num_blocks * row_width;
    let label_size = font.map(|f| label_rows_for_font(f) * row_width).unwrap_or(0);
    label_size + file_data_size
}

/// Check if a file fits before the next IDAT boundary.
fn file_fits_before_boundary(target_pos: usize, file_size: usize, row_width: usize) -> bool {
    let filtered_start = data_to_filtered_pos(target_pos, row_width);
    let filtered_end = data_to_filtered_pos(target_pos + file_size, row_width);
    !crosses_idat_boundary(filtered_start, filtered_end)
}

/// Build pixel data with variable row width and greedy IDAT bin-packing.
/// Files are placed in lexicographic order when possible, but smaller files
/// may be placed out of order to fill space before IDAT boundaries.
fn build_aligned_data(
    files: &[(&[u8], &[u8])],
    row_width: usize,
    font: Option<&BitmapFont>,
) -> (Vec<u8>, Vec<(Vec<u8>, Vec<u8>, usize, usize)>, HashSet<usize>) {
    let data_per_block = row_width - 4;
    let filtered_row_size = row_width + 1;

    let mut data = Vec::new();
    let mut entries = Vec::new();
    let mut final_block_rows = HashSet::new();

    // Pre-calculate sizes for all files
    let file_sizes: Vec<usize> = files.iter()
        .map(|(name, body)| calculate_file_size(name, body, row_width, font))
        .collect();

    // Track which files have been placed
    let mut placed = vec![false; files.len()];
    let mut num_placed = 0;

    while num_placed < files.len() {
        // Align to row boundary
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        let target_pos = data.len() + padding_to_row;

        // Find next unplaced file in lexicographic order
        let next_lex = placed.iter().position(|&p| !p).unwrap();

        // Check if it fits before the next IDAT boundary
        let file_to_place = if file_fits_before_boundary(target_pos, file_sizes[next_lex], row_width) {
            // Next file in lex order fits
            next_lex
        } else {
            // Scan forward for a file that fits
            let fitting_file = (next_lex + 1..files.len())
                .find(|&i| !placed[i] && file_fits_before_boundary(target_pos, file_sizes[i], row_width));

            if let Some(idx) = fitting_file {
                idx
            } else {
                // No file fits; skip to next boundary and place next lex file
                let new_target = next_boundary_aligned_pos(target_pos, row_width);
                data.resize(new_target, 0);
                next_lex
            }
        };

        // Place the selected file
        let (name, body) = &files[file_to_place];
        placed[file_to_place] = true;
        num_placed += 1;

        // Re-align to row boundary (may have changed if we skipped to boundary)
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        data.resize(data.len() + padding_to_row, 0);

        // Calculate header and extra field sizes
        let header_size = 30 + name.len();
        let bytes_used = header_size % row_width;
        let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };

        // Calculate number of deflate blocks and compressed size
        let num_blocks = if body.is_empty() { 1 } else { (body.len() + data_per_block - 1) / data_per_block };
        let compressed_size = num_blocks * filtered_row_size;

        // Pre-compute the header bytes (needed for label rendering)
        let crc = crc32(body);
        let header_bytes = build_local_header(name, body.len(), compressed_size, crc, extra_len);

        // Insert filename label if font is specified (with header as background)
        if let Some(f) = font {
            let label = render_filename_label(name, row_width, f, &header_bytes);
            data.extend_from_slice(&label);
        }

        let entry_start = data.len();

        // Calculate which row contains the final block
        let content_start = entry_start + header_size + extra_len;
        let last_block_pos = content_start + (num_blocks - 1) * row_width;
        let last_block_row = last_block_pos / row_width;
        final_block_rows.insert(last_block_row);

        entries.push((name.to_vec(), body.to_vec(), entry_start, compressed_size));

        // Write the pre-computed header
        data.extend_from_slice(&header_bytes);

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

/// Build ZIP local file header bytes (30 bytes + name + extra).
/// Returns the header as a Vec<u8> for use in label rendering.
fn build_local_header(
    name: &[u8],
    uncompressed_size: usize,
    compressed_size: usize,
    crc: u32,
    extra_len: usize,
) -> Vec<u8> {
    let mut header = Vec::with_capacity(30 + name.len() + extra_len);

    // Standard 30-byte ZIP local file header
    header.extend_from_slice(b"PK\x03\x04");                           // 0-3: signature
    header.extend_from_slice(&20_u16.to_le_bytes());                   // 4-5: version needed
    header.extend_from_slice(&0_u16.to_le_bytes());                    // 6-7: flags
    header.extend_from_slice(&8_u16.to_le_bytes());                    // 8-9: compression (deflate)
    header.extend_from_slice(&0_u16.to_le_bytes());                    // 10-11: mod time
    header.extend_from_slice(&0_u16.to_le_bytes());                    // 12-13: mod date
    header.extend_from_slice(&crc.to_le_bytes());                      // 14-17: CRC-32
    header.extend_from_slice(&(compressed_size as u32).to_le_bytes()); // 18-21
    header.extend_from_slice(&(uncompressed_size as u32).to_le_bytes()); // 22-25
    header.extend_from_slice(&(name.len() as u16).to_le_bytes());      // 26-27: name length
    header.extend_from_slice(&(extra_len as u16).to_le_bytes());       // 28-29: extra length
    header.extend_from_slice(name);                                     // 30+: filename

    // Extra field for alignment padding
    if extra_len > 0 {
        if extra_len >= 4 {
            // Valid extra field structure: ID + size + data
            header.extend_from_slice(&0x0000_u16.to_le_bytes()); // header ID
            header.extend_from_slice(&((extra_len - 4) as u16).to_le_bytes()); // data size
            header.resize(header.len() + extra_len - 4, 0); // data (zeros)
        } else {
            // Just padding bytes
            header.resize(header.len() + extra_len, 0);
        }
    }

    header
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
        // Small content should get minimum width (aligned to 64)
        assert_eq!(calculate_row_width(100, BASE_MIN_ROW_WIDTH), BASE_MIN_ROW_WIDTH);

        // 10KB: sqrt(10000) = 100 → aligned to 128
        let w = calculate_row_width(10_000, BASE_MIN_ROW_WIDTH);
        assert_eq!(w, 128, "Expected 128 (100 aligned to 64), got {}", w);

        // 40KB: sqrt(40000) ≈ 200 → aligned to 256
        let w = calculate_row_width(40_000, BASE_MIN_ROW_WIDTH);
        assert_eq!(w, 256, "Expected 256 (200 aligned to 64), got {}", w);

        // With larger min_width requirement (e.g., long filename) → aligned to 64
        let w = calculate_row_width(100, 60);
        assert_eq!(w, 64, "Expected min_width of 64 (60 aligned), got {}", w);

        // Verify alignment
        assert_eq!(w % ROW_WIDTH_ALIGNMENT, 0, "Width should be aligned to {}", ROW_WIDTH_ALIGNMENT);
    }

    #[test]
    fn test_min_row_width_for_files() {
        // Short filenames - returns BASE_MIN_ROW_WIDTH (64)
        let files = vec![(b"a.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files);
        assert_eq!(min, BASE_MIN_ROW_WIDTH); // 30 + 5 + 4 = 39 < 64, so 64

        // Long filename that requires wider rows
        let files = vec![(b"this-is-a-very-long-filename.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files);
        // 30 + 32 + 4 = 66 → aligned to 128
        assert_eq!(min, 128, "Expected 128 (66 aligned to 64), got {}", min);

        // Verify alignment
        assert_eq!(min % ROW_WIDTH_ALIGNMENT, 0, "Width should be aligned to {}", ROW_WIDTH_ALIGNMENT);
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
        // Check central directory has 3 entries (note: PK signatures also appear in label rows now)
        let cd_count = result.windows(4).filter(|w| *w == b"PK\x01\x02").count();
        assert_eq!(cd_count, 3, "Expected 3 central directory entries");
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

        // Check central directory has 5 entries (note: PK signatures also appear in label rows now)
        let cd_count = result.windows(4).filter(|w| *w == b"PK\x01\x02").count();
        assert_eq!(cd_count, 5, "Expected 5 central directory entries, got {}", cd_count);

        // Check EOCD exists
        assert!(result.windows(4).any(|w| w == b"PK\x05\x06"));
    }
}

    #[test]
    fn test_label_has_header_background() {
        use crate::checksums::crc32;
        let name = b"test.txt";
        let body = b"Hello!";
        let row_width = 64;

        let header_size = 30 + name.len();
        let bytes_used = header_size % row_width;
        let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };
        let compressed_size = row_width + 1;
        let crc = crc32(body);

        let header = build_local_header(name, body.len(), compressed_size, crc, extra_len);

        // Header should start with PK signature
        assert_eq!(&header[0..4], b"PK\x03\x04", "Header should start with PK signature");
        assert_eq!(header.len(), row_width, "Header should be exactly row_width bytes");

        // Get a font for testing
        let font = fonts::select_font(100).expect("Should get a font for small size");

        let label = render_filename_label(name, row_width, font, &header);

        // First row (padding above) should contain header bytes including PK signature
        assert_eq!(&label[0..4], b"PK\x03\x04", "Label should have PK signature in first row");
    }
