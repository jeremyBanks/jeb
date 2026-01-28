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

#[cfg(any(test, feature = "dev-dependencies"))]
mod validate;
#[cfg(any(test, feature = "dev-dependencies"))]
pub use validate::{validate_polyglot, assert_valid_polyglot, assert_valid_polyglot_with, ValidationResult, Expectations};

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

/// Deflate stored block header overhead (LEN + NLEN = 4 bytes).
/// Each row contains this header followed by the actual file data.
pub(crate) const DEFLATE_HEADER_OVERHEAD: usize = 4;

/// Alignment for the file data portion of each row.
/// Each row contains 64 bytes of actual ZIP file data.
pub(crate) const DATA_ALIGNMENT: usize = 64;

/// Base minimum row width (64 bytes data + 4 bytes deflate header = 68).
const BASE_MIN_ROW_WIDTH: usize = DATA_ALIGNMENT + DEFLATE_HEADER_OVERHEAD;

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

/// Check if a file's padding will require an internal terminator (BFINAL=1 in last content row).
/// This happens when (row_width - 4 - last_chunk_size) is not divisible by 5.
fn needs_internal_terminator(body_len: usize, row_width: usize) -> bool {
    let data_per_block = row_width - 4;
    if body_len == 0 {
        // Empty file: padding = row_width - 4
        return (row_width - 4) % 5 != 0;
    }
    let last_chunk_size = if body_len % data_per_block == 0 {
        data_per_block // Last chunk is full
    } else {
        body_len % data_per_block
    };
    let padding_needed = row_width - 4 - last_chunk_size;
    padding_needed % 5 != 0
}

/// Create a terminator row for ending a file's DEFLATE stream.
/// This row contains the empty final DEFLATE block (BFINAL=1, LEN=0, NLEN=0xFFFF).
/// The filter byte (0x01 for Sub) is added separately by add_smart_filter_bytes.
///
/// With Sub filter, the decoded values are:
/// - decoded[0] = raw[0] = 0x00
/// - decoded[1] = raw[1] + decoded[0] = 0x00
/// - decoded[2] = raw[2] + decoded[1] = 0xFF
/// - decoded[3] = raw[3] + decoded[2] = 0xFE (0xFF + 0xFF wrapped)
/// - decoded[4..] = 0 (if we set raw[4] = 0x02, raw[5..] = 0)
fn create_terminator_row(row_width: usize) -> Vec<u8> {
    let mut row = vec![0u8; row_width];
    // DEFLATE empty final block: LEN=0, NLEN=0xFFFF
    row[0] = 0x00; // LEN low
    row[1] = 0x00; // LEN high
    row[2] = 0xFF; // NLEN low
    row[3] = 0xFF; // NLEN high
    // With Sub filter, running sum after byte 3 is 0xFE
    // To make byte 4 decode to 0: raw[4] = (0 - 0xFE) mod 256 = 0x02
    if row_width > 4 {
        row[4] = 0x02;
    }
    // Remaining bytes are 0, which decode to 0 under Sub filter
    row
}

/// Format a file size with max 3 significant digits.
/// Examples: 8B, 48B, 999B, 1.00KiB, 49.5KiB, 495KiB, 1.00MiB, 49.5MiB
fn format_file_size(size: usize) -> String {
    if size < 1024 {
        // 0B to 1023B (always show bytes for < 1 KiB)
        format!("{}B", size)
    } else if size < 10 * 1024 {
        // 1.00KiB to 9.99KiB
        format!("{:.2}KiB", size as f64 / 1024.0)
    } else if size < 100 * 1024 {
        // 10.0KiB to 99.9KiB
        format!("{:.1}KiB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 {
        // 100KiB to 1023KiB
        format!("{}KiB", size / 1024)
    } else if size < 10 * 1024 * 1024 {
        // 1.00MiB to 9.99MiB
        format!("{:.2}MiB", size as f64 / (1024.0 * 1024.0))
    } else if size < 100 * 1024 * 1024 {
        // 10.0MiB to 99.9MiB
        format!("{:.1}MiB", size as f64 / (1024.0 * 1024.0))
    } else {
        // 100MiB+
        format!("{}MiB", size / (1024 * 1024))
    }
}

/// Render filename label rows using the specified font.
/// The meaningful header bytes (30 + filename) are "stretched" into each label row as a background,
/// then text is rendered on top with a 1-pixel halo erased for readability.
/// Extra field padding bytes are left as zeros - only actual ZIP metadata is shown.
/// Kerning checks against the accumulated rendering to avoid touching earlier chars.
///
/// If space permits (8px minimum gap), also renders file size right-aligned.
///
/// If `is_terminator` is true, the first row becomes a terminator row for the previous file:
/// - Bytes 0-3 are the DEFLATE empty final block (LEN=0, NLEN=0xFFFF)
/// - Bytes 4+ are pre-filtered for Sub filter to decode to black (0)
/// - The rest of the label rows use None filter as normal
fn render_filename_label(name: &[u8], row_width: usize, font: &BitmapFont, header_row: &[u8], is_terminator: bool, file_size: usize) -> Vec<u8> {
    let label_rows = label_rows_for_font(font);
    let mut result = Vec::with_capacity(label_rows * row_width);

    // Convert name to chars and get glyph lookups (with fallback chain)
    let chars: Vec<char> = name.iter().map(|&b| b as char).collect();
    let lookups: Vec<Option<fonts::GlyphLookup<'_>>> = chars.iter()
        .map(|&c| font.get_glyph(c))
        .collect();

    // Build accumulated canvas for kerning (checks against ALL previous chars, not just one)
    // Canvas must be wide enough for the entire text even if it overflows the display
    let max_canvas_width = (name.len() * font.width).max(row_width * 2);
    let mut canvas: Vec<Vec<bool>> = vec![vec![false; max_canvas_width]; font.height];
    let mut char_positions: Vec<(usize, i32)> = Vec::new();
    let mut total_width = 0i32;
    let mut rightmost_pixel = 0i32;

    for (i, lookup_opt) in lookups.iter().enumerate() {
        if let Some(lookup) = lookup_opt {
            let glyph = lookup.glyph;

            // Default position: after previous char
            let mut best_offset = total_width;

            // Apply kerning only if skip_kerning is false
            if !lookup.skip_kerning {
                best_offset = total_width + font.width as i32; // Start at default spacing

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
            }

            char_positions.push((i, best_offset));

            // Add glyph to canvas at best_offset (even for spaces, to track position)
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

    // Calculate starting x position
    // Default: 4px margin from left edge
    // If text overflows: right-align, but always leave 1px at right edge for halo
    const MARGIN: usize = 4;
    const RIGHT_PADDING: usize = 1; // Always leave 1px at right for halo
    const MIN_GAP: usize = 8; // Minimum gap between filename and size
    let usable_width = row_width - RIGHT_PADDING;

    let start_x: i32 = if actual_width + MARGIN <= usable_width {
        MARGIN as i32 // Fits with margin
    } else {
        // Right-align: text ends 1px before right edge, may truncate from left
        (usable_width as i32 - actual_width as i32).max(-(actual_width as i32))
    };

    // Calculate where filename ends (for gap calculation)
    // If start_x is negative, filename overflows left edge - use row_width as end to disable size
    let filename_end_x = if start_x >= 0 {
        (start_x as usize).saturating_add(actual_width)
    } else {
        row_width // Filename overflows, no room for size
    };

    // Try to render file size right-aligned if there's enough space
    let size_str = format_file_size(file_size);
    let size_chars: Vec<char> = size_str.chars().collect();
    let size_lookups: Vec<Option<fonts::GlyphLookup<'_>>> = size_chars.iter()
        .map(|&c| font.get_glyph(c))
        .collect();

    // Calculate size text width using same kerning logic
    let mut size_canvas: Vec<Vec<bool>> = vec![vec![false; max_canvas_width]; font.height];
    let mut size_char_positions: Vec<(usize, i32)> = Vec::new();
    let mut size_total_width = 0i32;
    let mut size_rightmost_pixel = 0i32;

    for (i, lookup_opt) in size_lookups.iter().enumerate() {
        if let Some(lookup) = lookup_opt {
            let glyph = lookup.glyph;
            let mut best_offset = size_total_width;

            if !lookup.skip_kerning {
                best_offset = size_total_width + font.width as i32;
                for test_offset in (size_total_width - font.width as i32 + 1)..=best_offset {
                    if test_offset < 0 { continue; }
                    let mut touches = false;
                    'check_size: for (gy, glyph_row) in glyph.iter().enumerate() {
                        for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                            if !pixel_on { continue; }
                            let cx = test_offset as usize + gx;
                            for dy in -1i32..=1 {
                                for dx in -1i32..=1 {
                                    let ny = gy as i32 + dy;
                                    let nx = cx as i32 + dx;
                                    if ny >= 0 && (ny as usize) < font.height && nx >= 0 {
                                        if size_canvas[ny as usize].get(nx as usize).copied().unwrap_or(false) {
                                            touches = true;
                                            break 'check_size;
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
            }

            size_char_positions.push((i, best_offset));

            for (gy, glyph_row) in glyph.iter().enumerate() {
                for (gx, &pixel_on) in glyph_row.iter().enumerate() {
                    if pixel_on {
                        let cx = best_offset as usize + gx;
                        if cx < max_canvas_width {
                            size_canvas[gy][cx] = true;
                        }
                        size_rightmost_pixel = size_rightmost_pixel.max(cx as i32 + 1);
                    }
                }
            }
            size_total_width = best_offset + font.width as i32;
        }
    }

    let size_actual_width = size_rightmost_pixel.max(0) as usize;

    // Check if size fits with minimum gap
    let size_start_x = usable_width as i32 - size_actual_width as i32 - MARGIN as i32;
    let render_size = size_start_x >= 0 && size_start_x as usize >= filename_end_x.saturating_add(MIN_GAP);

    // First, render text to a temporary bitmap to know where pixels are
    let mut text_bitmap: Vec<Vec<bool>> = vec![vec![false; row_width]; label_rows];

    // Padding row above (row 0) has no text
    // Text rows are 1..=font.height
    // Padding row below is font.height + 1

    // Render filename
    for text_row in 0..font.height {
        let result_row = text_row + 1; // +1 for padding above
        for &(char_idx, char_x) in &char_positions {
            if let Some(lookup) = &lookups[char_idx] {
                let glyph = lookup.glyph;
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

    // Render file size (right-aligned) if there's enough space
    if render_size {
        for text_row in 0..font.height {
            let result_row = text_row + 1;
            for &(char_idx, char_x) in &size_char_positions {
                if let Some(lookup) = &size_lookups[char_idx] {
                    let glyph = lookup.glyph;
                    if text_row < glyph.len() {
                        for (px, &pixel_on) in glyph[text_row].iter().enumerate() {
                            let x = (size_start_x + char_x + px as i32) as usize;
                            if x < row_width && pixel_on {
                                text_bitmap[result_row][x] = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Header "stretches" up from bottom row, but stops when it hits the halo
    // Only show meaningful header bytes (30 + filename), not extra field padding
    let meaningful_header_len = (30 + name.len()).min(header_row.len());

    // Track per-column whether the stretch is still active
    let mut stretch_active: Vec<bool> = vec![true; row_width];
    let mut rows_data: Vec<Vec<u8>> = vec![vec![0u8; row_width]; label_rows];

    // Process from bottom to top (header stretches upward)
    for row_idx in (0..label_rows).rev() {
        for x in 0..row_width {
            if !stretch_active[x] {
                continue; // Already blocked in this column
            }

            // Check if this position is in the halo (adjacent to text)
            let mut near_text = false;
            'halo: for dy in -1i32..=1 {
                for dx in -1i32..=1 {
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
                // Block this column from stretching further up
                stretch_active[x] = false;
            } else if x < meaningful_header_len {
                // Stretch continues - copy meaningful header byte only
                rows_data[row_idx][x] = header_row[x];
            }
            // For x >= meaningful_header_len, leave as 0 (don't show padding bytes)
        }
    }

    // Draw text pixels on top (white)
    for row_idx in 0..label_rows {
        for x in 0..row_width {
            if text_bitmap[row_idx][x] {
                rows_data[row_idx][x] = 0xFF;
            }
        }
    }

    // If this is a terminator label, transform the first row for Sub filter
    if is_terminator {
        // First row becomes terminator row with DEFLATE header
        // Bytes 0-3: empty final block (LEN=0, NLEN=0xFFFF)
        // Bytes 4+: pre-filtered so Sub filter decodes to the desired visual (black)
        rows_data[0][0] = 0x00; // LEN low
        rows_data[0][1] = 0x00; // LEN high
        rows_data[0][2] = 0xFF; // NLEN low
        rows_data[0][3] = 0xFF; // NLEN high
        // With Sub filter, running sum after byte 3 is 0xFE
        // To make byte 4 decode to 0: raw[4] = (0 - 0xFE) mod 256 = 0x02
        if row_width > 4 {
            rows_data[0][4] = 0x02;
            // Remaining bytes stay 0, which decode to 0 under Sub filter
            for x in 5..row_width {
                rows_data[0][x] = 0x00;
            }
        }
    }

    // Output all rows
    for row_idx in 0..label_rows {
        result.extend_from_slice(&rows_data[row_idx]);
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

/// Round up so that the data portion (row_width - 4) is a multiple of 64.
/// This ensures each row contains a whole number of 64-byte data chunks.
fn align_to_row_width(value: usize) -> usize {
    let min_data = value.saturating_sub(DEFLATE_HEADER_OVERHEAD);
    let aligned_data = ((min_data + DATA_ALIGNMENT - 1) / DATA_ALIGNMENT) * DATA_ALIGNMENT;
    aligned_data.max(DATA_ALIGNMENT) + DEFLATE_HEADER_OVERHEAD
}

/// Calculate optimal row width for approximately square images.
/// Returns width such that height >= width (portrait/square orientation).
/// The data portion (row_width - 4) is always a multiple of 64 bytes.
///
/// The `min_width` parameter ensures the row is wide enough to fit ZIP headers
/// without spanning multiple rows (which would corrupt them with filter bytes).
fn calculate_row_width(total_data_estimate: usize, min_width: usize) -> usize {
    // For height >= width, we need: total_data / width >= width
    // Therefore: width <= sqrt(total_data)
    // Using floor ensures height >= width
    let ideal = (total_data_estimate as f64).sqrt();
    let width = ideal.floor() as usize;

    // Clamp to minimum safe width and align data portion to DATA_ALIGNMENT
    align_to_row_width(width.max(min_width))
}

/// Calculate minimum row width needed for a set of files.
/// Ensures ZIP local file headers (30 bytes + filename) fit in one row.
/// Returns a value where (row_width - 4) is a multiple of 64.
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

    // Hash all file paths and contents for deterministic font/palette selection
    let mut hash_input: Vec<u8> = Vec::new();
    for (path, content) in files {
        hash_input.extend_from_slice(path);
        hash_input.push(0); // separator
        hash_input.extend_from_slice(content);
        hash_input.push(0); // separator
    }
    let content_hash = crc32(&hash_input);

    // Select font based on content size and hash (None if too large for labels)
    let font = select_font(total_content_size, content_hash);

    // Two-pass approach for optimal dimensions:
    // Pass 1: Build with estimated width to get actual data size
    // Pass 2: Rebuild with width calculated from actual size

    // Pass 1: Use estimate for initial build
    let estimated_size = estimate_total_size(files, font);
    let initial_width = calculate_row_width(estimated_size, min_width);
    let (initial_data, _, _) = build_aligned_data(files, initial_width, font);

    // Pass 2: Calculate optimal width from actual size
    let actual_size = initial_data.len();
    let mut row_width = calculate_row_width(actual_size, min_width);

    // Optimization: If all files would fit in one content row each, use narrower width.
    // This matters for many small files where the "square" heuristic wastes space.
    let max_body_len = files.iter().map(|(_, body)| body.len()).max().unwrap_or(0);
    // For one content row: body must fit in (row_width - 4) data area
    // So min width = max_body + 4, then align to DATA_ALIGNMENT
    let min_width_for_single_row = align_to_row_width((max_body_len + DEFLATE_HEADER_OVERHEAD).max(min_width));

    if min_width_for_single_row < row_width {
        // Verify: at this width, do all files fit in one content row?
        let data_per_row = min_width_for_single_row - DEFLATE_HEADER_OVERHEAD;
        let all_fit = files.iter().all(|(_, body)| body.len() <= data_per_row);
        if all_fit {
            row_width = min_width_for_single_row;
        }
    }

    // Build with the (possibly optimized) row_width
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

    // Debug: print terminator rows for file_045
    // Find file_045 entry
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

/// Calculate the total size a file will occupy (label + header + content + terminator).
/// Note: When labels are enabled, the terminator may merge with the next file's label,
/// but we count it conservatively for bin packing purposes.
fn calculate_file_size(name: &[u8], body: &[u8], row_width: usize, font: Option<&BitmapFont>) -> usize {
    let data_per_block = row_width - 4;
    let header_size = 30 + name.len();
    let bytes_for_alignment = header_size % row_width;
    let extra_len = if bytes_for_alignment == 0 { 0 } else { row_width - bytes_for_alignment };
    let num_blocks = if body.is_empty() { 1 } else { (body.len() + data_per_block - 1) / data_per_block };
    let file_data_size = header_size + extra_len + num_blocks * row_width;
    let label_size = font.map(|f| label_rows_for_font(f) * row_width).unwrap_or(0);
    // Add terminator row only if not using internal terminator
    let uses_internal = needs_internal_terminator(body.len(), row_width);
    let terminator_size = if uses_internal { 0 } else { row_width };
    label_size + file_data_size + terminator_size
}

/// Check if a file fits before the next IDAT boundary.
/// Build pixel data with bin packing, preferred order, and even spacing.
///
/// Algorithm (from specification):
/// 1. Constrained packing: Process in preferred order, but allow nearby-in-size
///    items (within 12.5% of largest remaining) to jump ahead for better fit.
/// 2. Relaxation check: After each placement, simulate if remaining items fit.
///    If yes, switch to relaxed mode.
/// 3. Relaxed packing: Place remaining items in strict preferred order.
/// 4. Spacing: Distribute leftover space evenly within each full IDAT bucket.
///    Edge gaps (before first, after last) have half weight.
fn build_aligned_data(
    files: &[(&[u8], &[u8])],
    row_width: usize,
    font: Option<&BitmapFont>,
) -> (Vec<u8>, Vec<(Vec<u8>, Vec<u8>, usize, usize)>, HashSet<usize>) {
    let data_per_block = row_width - 4;
    let filtered_row_size = row_width + 1;

    if files.is_empty() {
        return (Vec::new(), Vec::new(), HashSet::new());
    }

    // Pre-calculate sizes for all files
    let file_sizes: Vec<usize> = files.iter()
        .map(|(name, body)| calculate_file_size(name, body, row_width, font))
        .collect();

    // Phase 1: Bin packing (largest-first worst-fit)
    let mut bucket_assignments = bin_pack_largest_first(files, &file_sizes, row_width);

    // Phase 2: Lexicographic sorting
    // Sort files within each bucket by filename
    for bucket in &mut bucket_assignments {
        bucket.sort_by(|&a, &b| files[a].0.cmp(files[b].0));
    }

    // Sort all buckets except the last one by their first filename
    // (last bucket stays last because it gets truncated/no spacing)
    if bucket_assignments.len() > 1 {
        let last_bucket = bucket_assignments.pop().unwrap();
        bucket_assignments.sort_by(|a, b| {
            let first_a = a.first().map(|&i| files[i].0);
            let first_b = b.first().map(|&i| files[i].0);
            first_a.cmp(&first_b)
        });
        bucket_assignments.push(last_bucket);
    }

    // Phase 3: Calculate spacing for each bucket
    let bucket_spacing = calculate_bucket_spacing(&bucket_assignments, &file_sizes, row_width);

    // Phase 4: Place files with pre-calculated spacing
    // Each file's DEFLATE stream needs a terminator row (BFINAL=1).
    // - If the next file has a label with no spacing gap, the label's first row is the terminator
    // - Otherwise, add an explicit terminator row after the content
    let mut data = Vec::new();
    let mut entries = Vec::new();
    let mut terminator_rows = HashSet::new();

    // Flatten bucket assignments to get the order of files with their spacing info
    let mut file_order_with_spacing: Vec<(usize, usize)> = Vec::new(); // (file_idx, spacing_rows)
    for (bucket_idx, file_indices) in bucket_assignments.iter().enumerate() {
        let spacing = &bucket_spacing[bucket_idx];
        for (file_in_bucket, &file_idx) in file_indices.iter().enumerate() {
            file_order_with_spacing.push((file_idx, spacing[file_in_bucket]));
        }
    }

    // Track if previous file needs a terminator
    let mut pending_terminator = false;

    for (order_idx, &(file_idx, spacing_rows)) in file_order_with_spacing.iter().enumerate() {
        let has_label = font.is_some();
        let has_spacing_gap = spacing_rows > 0;

        // Determine if this file's label can serve as terminator for previous file.
        // The label can only serve as terminator if it's placed IMMEDIATELY after the
        // previous file's content (same row). This requires:
        // 1. Previous file has pending_terminator = true
        // 2. This file has a label (font is Some)
        // 3. No spacing gap (spacing_rows == 0)
        // 4. No IDAT boundary crossing (would insert padding between files)
        //
        // We need to check boundary crossing BEFORE deciding, because if crossing happens,
        // there will be padding rows between the previous file and this label.
        let mut label_is_terminator = pending_terminator && has_label && !has_spacing_gap;

        // Check if IDAT boundary crossing will happen - this adds padding that breaks
        // the terminator chain. Calculate where the file would be placed.
        if label_is_terminator {
            // Simulate the position after alignment (spacing_rows is 0 here)
            let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
            let pos_after_align = data.len() + padding_to_row;

            // Check if this file would cross an IDAT boundary
            let file_size = file_sizes[file_idx];
            let start_filtered = data_to_filtered_pos(pos_after_align, row_width);
            let end_filtered = data_to_filtered_pos(pos_after_align + file_size, row_width);
            if crosses_idat_boundary(start_filtered, end_filtered) {
                // Boundary crossing will happen - label can't serve as terminator
                label_is_terminator = false;
            }
        }

        // If previous file needs terminator and we can't use this label, add explicit one
        if pending_terminator && !label_is_terminator {
            let terminator_row = data.len() / row_width;
            terminator_rows.insert(terminator_row);
            let terminator = create_terminator_row(row_width);
            data.extend_from_slice(&terminator);
        }
        // Note: pending_terminator will be set at the end of this iteration

        // Add spacing BEFORE this file (after any terminator)
        if spacing_rows > 0 {
            // Align to row first, then add spacing
            let padding = (row_width - (data.len() % row_width)) % row_width;
            data.resize(data.len() + padding + spacing_rows * row_width, 0);
        }

        // Align to row boundary
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        data.resize(data.len() + padding_to_row, 0);

        // Check if this file would cross an IDAT boundary
        let file_size = file_sizes[file_idx];
        let start_filtered = data_to_filtered_pos(data.len(), row_width);
        let end_filtered = data_to_filtered_pos(data.len() + file_size, row_width);
        if crosses_idat_boundary(start_filtered, end_filtered) {
            // Pad to next boundary-aligned position
            let boundary_target = next_boundary_aligned_pos(data.len(), row_width);
            data.resize(boundary_target, 0);
        }

        // Place the file
        let (name, body) = &files[file_idx];

        // Calculate header and extra field sizes
        let header_size = 30 + name.len();
        let bytes_used = header_size % row_width;
        let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };

        // Calculate number of deflate blocks and compressed size
        // Add +1 row for terminator ONLY if padding divides evenly by 5 (separate terminator row)
        // Otherwise, the last content row has BFINAL=1 (internal terminator)
        let num_content_blocks = if body.is_empty() { 1 } else { (body.len() + data_per_block - 1) / data_per_block };
        let uses_internal_terminator = needs_internal_terminator(body.len(), row_width);
        let terminator_rows_count = if uses_internal_terminator { 0 } else { 1 };
        let compressed_size = (num_content_blocks + terminator_rows_count) * filtered_row_size;

        // Pre-compute the header bytes
        let crc = crc32(body);
        let header_bytes = build_local_header(name, body.len(), compressed_size, crc, extra_len);

        // Insert filename label if font is specified
        if let Some(f) = font {
            if label_is_terminator {
                // This label's first row serves as terminator for previous file
                let terminator_row = data.len() / row_width;
                terminator_rows.insert(terminator_row);
            }
            let label = render_filename_label(name, row_width, f, &header_bytes, label_is_terminator, body.len());
            data.extend_from_slice(&label);
        }

        let entry_start = data.len();

        entries.push((name.to_vec(), body.to_vec(), entry_start, compressed_size));

        // Write the header and content
        data.extend_from_slice(&header_bytes);
        let mut needs_internal_terminator = false;
        let deflate_content = encode_as_deflate_blocks(body, row_width, &mut needs_internal_terminator);
        data.extend_from_slice(&deflate_content);

        if needs_internal_terminator {
            // Padding didn't divide evenly by 5, so the last content row
            // has BFINAL=1 with anti-Sub padding (no separate terminator needed)
            let last_content_row = (data.len() - 1) / row_width;
            terminator_rows.insert(last_content_row);
            pending_terminator = false;
        } else {
            // This file needs a separate terminator row
            pending_terminator = true;
        }
    }

    // Handle terminator for the very last file (if it used empty-block padding)
    if pending_terminator {
        let terminator_row = data.len() / row_width;
        terminator_rows.insert(terminator_row);
        let terminator = create_terminator_row(row_width);
        data.extend_from_slice(&terminator);
    }

    (data, entries, terminator_rows)
}

/// Calculate spacing for each bucket.
/// Full buckets get even spacing with half-weight edges.
/// First bucket has no leading gap (weight 0 instead of 0.5).
/// Last bucket gets no spacing (content packed tight).
fn calculate_bucket_spacing(
    bucket_assignments: &[Vec<usize>],
    file_sizes: &[usize],
    row_width: usize,
) -> Vec<Vec<usize>> {
    let filtered_row_size = row_width + 1;
    let rows_per_bucket = IDAT_BLOCK_SIZE / filtered_row_size;
    let bucket_capacity_bytes = rows_per_bucket * row_width;

    let num_buckets = bucket_assignments.len();
    let mut result = Vec::with_capacity(num_buckets);

    for (bucket_idx, file_indices) in bucket_assignments.iter().enumerate() {
        let is_first_bucket = bucket_idx == 0;
        let is_last_bucket = bucket_idx == num_buckets - 1;
        let num_files = file_indices.len();

        // No spacing for last bucket or single-bucket images
        if is_last_bucket || num_files == 0 {
            result.push(vec![0; num_files]);
            continue;
        }

        // Calculate total content bytes
        let total_content: usize = file_indices.iter()
            .map(|&idx| file_sizes[idx])
            .sum();

        let slack_bytes = bucket_capacity_bytes.saturating_sub(total_content);
        let slack_rows = slack_bytes / row_width;

        if slack_rows == 0 {
            result.push(vec![0; num_files]);
            continue;
        }

        // Distribute with half-weight edges: [0.5, 1, 1, ..., 1, 0.5] = N weight total
        // But first bucket: no leading gap, so [0, 1, 1, ..., 1, 0.5] = N - 0.5 weight
        let first_weight = if is_first_bucket { 0.0 } else { 0.5 };
        let total_weight = first_weight + (num_files - 1) as f64 + 0.5;
        let rows_per_unit = slack_rows as f64 / total_weight;

        let mut spacing = Vec::with_capacity(num_files);
        let mut allocated = 0usize;

        for i in 0..num_files {
            let weight = if i == 0 { first_weight } else { 1.0 };
            let rows = (weight * rows_per_unit).floor() as usize;
            spacing.push(rows);
            allocated += rows;
        }

        // Distribute remainder to middle gaps (skip first gap if first bucket)
        let mut remaining = slack_rows.saturating_sub(allocated);
        let start_idx = if is_first_bucket { 1 } else { 0 };
        for i in start_idx..num_files {
            if remaining == 0 { break; }
            spacing[i] += 1;
            remaining -= 1;
        }

        result.push(spacing);
    }

    result
}

/// Perform bin packing using largest-first worst-fit algorithm.
/// Worst-fit leaves more slack in each bucket for even spacing distribution.
/// Returns bucket assignments: Vec<Vec<usize>> where each inner Vec contains file indices.
fn bin_pack_largest_first(
    files: &[(&[u8], &[u8])],
    file_sizes: &[usize],
    row_width: usize,
) -> Vec<Vec<usize>> {
    let filtered_row_size = row_width + 1;
    let rows_per_bucket = IDAT_BLOCK_SIZE / filtered_row_size;
    let bucket_capacity = rows_per_bucket * row_width;

    let n = file_sizes.len();
    if n == 0 {
        return vec![];
    }

    // Create list of (file_idx, size) and sort by size descending, tiebreak by filename
    let mut sorted_by_size: Vec<usize> = (0..n).collect();
    sorted_by_size.sort_by(|&a, &b| {
        file_sizes[b].cmp(&file_sizes[a])
            .then_with(|| files[a].0.cmp(files[b].0))
    });

    // bucket_remaining[i] = remaining space in bucket i
    let mut bucket_remaining: Vec<usize> = vec![];
    // bucket_contents[i] = list of file indices in bucket i
    let mut bucket_contents: Vec<Vec<usize>> = vec![];

    for file_idx in sorted_by_size {
        let size = file_sizes[file_idx];

        // Find bucket with MOST remaining space that still fits the file (worst-fit)
        // This spreads files more evenly, leaving slack for spacing
        let mut best_bucket: Option<usize> = None;
        let mut best_remaining = 0usize;

        for (bucket_idx, &remaining) in bucket_remaining.iter().enumerate() {
            if remaining >= size && remaining > best_remaining {
                best_bucket = Some(bucket_idx);
                best_remaining = remaining;
            }
        }

        match best_bucket {
            Some(bucket_idx) => {
                bucket_remaining[bucket_idx] -= size;
                bucket_contents[bucket_idx].push(file_idx);
            }
            None => {
                // Create new bucket
                let capacity = if size > bucket_capacity { size } else { bucket_capacity };
                bucket_remaining.push(capacity - size);
                bucket_contents.push(vec![file_idx]);
            }
        }
    }

    bucket_contents
}

/// Encode data as deflate stored blocks with variable row width.
/// All content rows use None filter (0x00) - terminator rows are added separately.
///
/// IMPORTANT: Padding after actual data must be valid DEFLATE blocks, because
/// content rows have BFINAL=0 and the decoder continues reading after the data.
/// We fill padding with empty stored blocks (5 bytes each: 00 00 00 FF FF).
/// If padding doesn't divide evenly by 5, the last content row uses BFINAL=1
/// Encode body as DEFLATE stored blocks, one per row.
/// All content rows use BFINAL=0. A separate terminator row provides BFINAL=1.
///
/// Padding after content is filled with empty stored blocks (5 bytes each).
/// If padding doesn't divide evenly by 5, uses internal terminator approach
/// where the last content row has BFINAL=1 (causes some visual artifacts).
fn encode_as_deflate_blocks(body: &[u8], row_width: usize, needs_internal_terminator: &mut bool) -> Vec<u8> {
    let data_per_block = row_width - 4;
    let mut result = Vec::new();

    *needs_internal_terminator = false;

    if body.is_empty() {
        // Empty file: single stored block with 0 length
        result.extend_from_slice(&0_u16.to_le_bytes());
        result.extend_from_slice(&0xFFFF_u16.to_le_bytes());
        // Fill padding with empty stored blocks
        let padding_needed = row_width - 4;
        fill_padding_with_empty_blocks(&mut result, padding_needed, row_width, needs_internal_terminator);
        return result;
    }

    let chunks: Vec<_> = body.chunks(data_per_block).collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        let len = chunk.len() as u16;
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&len.not().to_le_bytes());
        result.extend_from_slice(chunk);

        let block_size = 4 + chunk.len();
        if block_size < row_width {
            let padding_needed = row_width - block_size;

            if is_last {
                fill_padding_with_empty_blocks(&mut result, padding_needed, row_width, needs_internal_terminator);
            } else {
                // Non-last rows with partial data - shouldn't happen normally
                result.resize(result.len() + padding_needed, 0);
            }
        }
    }

    result
}

/// Fill padding with valid empty DEFLATE stored blocks.
/// Each empty block is 5 bytes: [BFINAL=0, BTYPE=0][LEN=0][NLEN=0xFFFF]
/// If padding doesn't divide evenly by 5, uses internal terminator approach
/// (sets needs_internal_terminator = true and fills with anti-Sub padding).
///
/// # Anti-Sub Padding
/// When internal terminator is used, the row gets Sub filter (0x01). Under Sub
/// filter, decoded[i] = raw[i] + decoded[i-1], which causes visual cumulative
/// sums. To make padding decode to black (0), we compute:
/// - First padding byte: (256 - running_sum) mod 256 to cancel the accumulated sum
/// - Remaining bytes: 0 (decoded[i] = 0 + 0 = 0)
fn fill_padding_with_empty_blocks(result: &mut Vec<u8>, padding_needed: usize, row_width: usize, needs_internal_terminator: &mut bool) {
    if padding_needed == 0 {
        return;
    }

    if padding_needed % 5 == 0 {
        // Perfect fit: fill with empty stored blocks
        let num_blocks = padding_needed / 5;
        for _ in 0..num_blocks {
            result.push(0x00); // BFINAL=0, BTYPE=0
            result.extend_from_slice(&0_u16.to_le_bytes()); // LEN=0
            result.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // NLEN=0xFFFF
        }
    } else {
        // Can't fill evenly: this row needs internal terminator (BFINAL=1)
        // The filter byte becomes 0x01 (Sub filter = BFINAL=1)
        *needs_internal_terminator = true;

        // Calculate the running sum of all bytes in this row so far.
        // The row contains [LEN][NLEN][content][padding], and we need to sum
        // everything before padding, which is (row_width - padding_needed) bytes.
        // Since result already contains all bytes up to the padding point,
        // row_start = result.len() - (row_width - padding_needed).
        let bytes_before_padding = row_width - padding_needed;
        let row_start = result.len().saturating_sub(bytes_before_padding);
        let mut running_sum: u8 = 0;
        for &byte in &result[row_start..] {
            running_sum = running_sum.wrapping_add(byte);
        }

        // First padding byte cancels the running sum so decoded value is 0
        let anti_sum = (256u16 - running_sum as u16) as u8;
        result.push(anti_sum);

        // Remaining padding bytes are 0, which decode to 0 under Sub filter
        if padding_needed > 1 {
            result.resize(result.len() + padding_needed - 1, 0);
        }
    }
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
            header.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // header ID (0xFFFF = third-party use)
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
        // Small content should get minimum width (68 = 64 data + 4 header)
        assert_eq!(calculate_row_width(100, BASE_MIN_ROW_WIDTH), BASE_MIN_ROW_WIDTH);
        assert_eq!(BASE_MIN_ROW_WIDTH, 68);

        // 10KB: sqrt(10000) = 100, data portion aligned to 128 → row_width = 132
        let w = calculate_row_width(10_000, BASE_MIN_ROW_WIDTH);
        assert_eq!(w, 132, "Expected 132 (128 data + 4 header), got {}", w);

        // 40KB: sqrt(40000) ≈ 200, data portion aligned to 256 → row_width = 260
        let w = calculate_row_width(40_000, BASE_MIN_ROW_WIDTH);
        assert_eq!(w, 260, "Expected 260 (256 data + 4 header), got {}", w);

        // With smaller min_width requirement → still gets 68 minimum
        let w = calculate_row_width(100, 60);
        assert_eq!(w, 68, "Expected min_width of 68 (64 data aligned), got {}", w);

        // Verify data portion alignment (row_width - 4 should be multiple of 64)
        assert_eq!((w - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "Data portion should be aligned to {}", DATA_ALIGNMENT);
    }

    #[test]
    fn test_min_row_width_for_files() {
        // Short filenames - returns BASE_MIN_ROW_WIDTH (68)
        let files = vec![(b"a.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files);
        assert_eq!(min, BASE_MIN_ROW_WIDTH); // 30 + 5 + 4 = 39 < 68, so 68

        // Moderately long filename - still fits in 68
        let files = vec![(b"this-is-a-very-long-filename.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files);
        // 30 + 32 + 4 = 66 < 68, so still 68
        assert_eq!(min, 68, "Expected 68 (66 fits in minimum), got {}", min);

        // Very long filename that requires wider rows
        let files = vec![(b"this-is-an-extremely-long-filename-that-exceeds-minimum.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files);
        // 30 + 58 + 4 = 92 → data aligned to 128 → row_width = 132
        assert_eq!(min, 132, "Expected 132 (128 data + 4 header), got {}", min);

        // Verify data portion alignment
        assert_eq!((min - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "Data portion should be aligned to {}", DATA_ALIGNMENT);
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

        // Get a font for testing (pass 0 as hash since we just need any font)
        let font = fonts::select_font(100, 0).expect("Should get a font for small size");

        let label = render_filename_label(name, row_width, font, &header, false, body.len());

        // Bottom row (padding below text) should have header bytes - stretch starts there
        // label_rows = font.height + 2 = 5 for typical font
        let label_rows = font.height + 2;
        let last_row_start = (label_rows - 1) * row_width;
        assert_eq!(&label[last_row_start..last_row_start + 4], b"PK\x03\x04",
            "Label should have PK signature in bottom row (stretch starts there)");
    }
