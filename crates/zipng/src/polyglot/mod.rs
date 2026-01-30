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
//! ## Central Directory Embedding
//!
//! The ZIP central directory (CD) and End of Central Directory (EOCD)
//! are embedded directly as PNG pixel rows inside the IDAT chunk — not
//! as a trailing block after pixel data. Zero-valued fields in the CD
//! structure (disk_number, internal_attrs, external_attrs at offsets
//! 34-41) and EOCD (disk fields at offsets 4-7) provide valid PNG None
//! filter bytes (0x00) at row boundaries. Extra field padding is
//! inserted to align these zero fields with row boundaries when needed.
//! This results in zero bytes after IEND.
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

pub mod fonts;

#[cfg(any(test, feature = "dev-dependencies"))]
mod validate;
#[cfg(any(test, feature = "dev-dependencies"))]
pub use validate::{validate_polyglot, assert_valid_polyglot, assert_valid_polyglot_with, ValidationResult, Expectations};

use std::ops::Not;

use crate::checksums::{adler32, crc32};
use crate::io::{OutputBuffer, output_buffer};
use crate::png::write_png::{write_png_header, write_png_chunk, write_png_footer};
use fonts::{BitmapFont, FontSelection, select_font};

// Re-export types from png module with compatibility aliases
pub use crate::png::{BitDepth, ColorType};
pub use crate::png::BitDepth::*;
pub use crate::png::ColorType::*;

/// Alias for backward compatibility - ColorMode is now ColorType
pub type ColorMode = ColorType;

/// Alias for backward compatibility - Lightness is now Luminance
pub const LIGHTNESS: ColorType = ColorType::Luminance;
pub const LIGHTNESS_ALPHA: ColorType = ColorType::LuminanceAlpha;

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
///
/// This is a loose, conservative lower bound. The true maximum depends on
/// row width: `floor(65535 / (row_width + 1)) * (row_width - 4)`, which
/// for the minimum row width of 68 gives 60,736. This constant is a
/// round-down that should be safe for all row widths, but it has not been
/// rigorously verified and could be invalidated by future layout changes.
///
/// TODO: Replace this with a precise, row-width-aware check. The current
/// constant is fragile — it may be unnecessarily restrictive for wide rows
/// or (worse) insufficient if layout assumptions change.
pub const MAX_FILE_CONTENT_SIZE: usize = 60_000;

/// Check if two glyphs would touch at a given horizontal offset.
/// Returns true if any pixels are 8-directionally adjacent.
pub fn glyphs_touch(prev: &[Vec<bool>], next: &[Vec<bool>], offset: i32) -> bool {
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
pub fn min_glyph_gap(prev: &[Vec<bool>], next: &[Vec<bool>], glyph_width: usize) -> i32 {
    // Try negative gaps first (tighter kerning)
    for gap in (-(glyph_width as i32 - 1))..((glyph_width * 2) as i32) {
        let offset = glyph_width as i32 + gap;
        if offset > 0 && !glyphs_touch(prev, next, offset) {
            return gap;
        }
    }
    glyph_width as i32 // fallback: full glyph width gap
}

/// Calculate the number of label rows for a given font height.
fn label_rows_for_height(height: usize) -> usize {
    // 1 empty row above + font height + 1 empty row below
    1 + height + 1
}

/// Calculate the number of label rows for a font selection (uses the taller font).
fn label_rows_for_font(font: &FontSelection) -> usize {
    label_rows_for_height(font.max_height())
}

/// Check if a file's padding will require an internal terminator (BFINAL=1 in last content row).
/// This happens when (row_width - 4 - last_chunk_size) is not divisible by 5.
/// Returns the padding needed in the last row of a file's DEFLATE content.
fn last_row_padding(body_len: usize, row_width: usize) -> usize {
    let data_per_block = row_width - 4;
    if body_len == 0 {
        return row_width - 4;
    }
    let last_chunk_size = if body_len.is_multiple_of(data_per_block) {
        data_per_block
    } else {
        body_len % data_per_block
    };
    row_width - 4 - last_chunk_size
}

/// Returns true if the terminator will be embedded in the last row's padding.
/// This happens when padding is non-zero and divisible by 5.
/// When padding % 5 != 0, a separate terminator row is used instead.
fn embeds_terminator_in_padding(body_len: usize, row_width: usize) -> bool {
    let padding = last_row_padding(body_len, row_width);
    padding > 0 && padding.is_multiple_of(5)
}

/// Calculate the compressed size for a file's DEFLATE content.
/// When the terminator is embedded in padding, compressed_size excludes trailing
/// zero bytes (dead space) so only covers through the terminator block.
fn calculate_compressed_size(body_len: usize, row_width: usize) -> usize {
    let data_per_block = row_width - 4;
    let filtered_row_size = row_width + 1; // row_width data + 1 filter byte
    let num_content_blocks = if body_len == 0 { 1 } else { body_len.div_ceil(data_per_block) };

    if embeds_terminator_in_padding(body_len, row_width) {
        // Last row: only count through the terminator (5 bytes after content).
        // Preceding rows are full. Last row = filter(1) + header(4) + data + terminator(5).
        let last_chunk_len = if body_len == 0 { 0 } else {
            let remainder = body_len % data_per_block;
            if remainder == 0 { data_per_block } else { remainder }
        };
        (num_content_blocks - 1) * filtered_row_size + 1 + 4 + last_chunk_len + 5
    } else {
        // Separate terminator row (no padding, padding not divisible by 5, or zero padding).
        // The terminator row completes the DEFLATE stream. compressed_size includes
        // the full terminator row; any dead space after BFINAL is ignored by decompressors.
        (num_content_blocks + 1) * filtered_row_size
    }
}

/// Create a terminator row for ending a file's DEFLATE stream.
/// Uses None filter (0x00). The row data layout depends on `bridge_bytes`:
/// the number of padding bytes from the previous row that started a DEFLATE
/// stored block which spans into this row.
///
/// The filter byte (0x00) is added separately by `add_filter_bytes`.
/// It serves double duty as both PNG None filter and a DEFLATE byte:
///
/// | bridge | padding fill    | filter=  | row data starts with           |
/// |--------|-----------------|----------|--------------------------------|
/// | 0      | (all mod-5)     | BFINAL=0 | 00 00 FF FF 01 00 00 FF FF    |
/// | 1      | 00              | LEN_lo   | 00 FF FF 01 00 00 FF FF       |
/// | 2      | 00 00           | LEN_hi   | FF FF 01 00 00 FF FF          |
/// | 3      | 02 00 00        | LEN_hi   | FF FF 01 00 00 FF FF          |
/// | 4      | 02 08 00 00     | LEN_hi   | FF FF 01 00 00 FF FF          |
///
/// For bridge=0: filter starts a new empty non-final stored block, then data
///   completes it and provides the empty final stored block.
/// For bridge=1: previous row's `00` started a stored block, filter is LEN_lo.
/// For bridge=2: previous row's `00 00` = header + LEN_lo, filter is LEN_hi.
/// For bridge=3,4: previous row uses fixed Huffman block(s) to consume extra
///   bytes, ending with a stored block header + LEN_lo; filter is LEN_hi.
fn create_terminator_row(row_width: usize, bridge_bytes: usize) -> Vec<u8> {
    let mut row = vec![0u8; row_width];
    match bridge_bytes {
        0 => {
            // Filter byte 0x00 starts a non-final stored block (BFINAL=0, BTYPE=00)
            // Row data completes it (LEN=0, NLEN=0xFFFF), then provides final block
            row[0] = 0x00; // LEN low
            row[1] = 0x00; // LEN high
            row[2] = 0xFF; // NLEN low
            row[3] = 0xFF; // NLEN high
            row[4] = 0x01; // BFINAL=1, BTYPE=00 (stored)
            row[5] = 0x00; // LEN low
            row[6] = 0x00; // LEN high
            row[7] = 0xFF; // NLEN low
            row[8] = 0xFF; // NLEN high
        }
        1 => {
            // Filter byte 0x00 is LEN_lo. Row data provides LEN_hi + NLEN + final block.
            row[0] = 0x00; // LEN high → LEN=0
            row[1] = 0xFF; // NLEN low
            row[2] = 0xFF; // NLEN high
            row[3] = 0x01; // BFINAL=1, BTYPE=00
            row[4] = 0x00; // LEN low
            row[5] = 0x00; // LEN high
            row[6] = 0xFF; // NLEN low
            row[7] = 0xFF; // NLEN high
        }
        2 | 3 | 4 => {
            // Filter byte 0x00 is LEN_hi → LEN=0. Row data provides NLEN + final block.
            row[0] = 0xFF; // NLEN low
            row[1] = 0xFF; // NLEN high
            row[2] = 0x01; // BFINAL=1, BTYPE=00
            row[3] = 0x00; // LEN low
            row[4] = 0x00; // LEN high
            row[5] = 0xFF; // NLEN low
            row[6] = 0xFF; // NLEN high
        }
        _ => unreachable!("bridge_bytes must be 0..=4"),
    }
    row
}

/// Generate progressively shorter file size string candidates by truncating
/// trailing decimals (rounding up if any truncated digits are non-zero).
/// Returns candidates from most precise to least precise.
///
/// Example for 1,168 bytes (1.14KiB):
///   ["1.14KiB", "1.2KiB", "2KiB"]
fn format_file_size_candidates(size: usize) -> Vec<String> {
    let full = format_file_size(size);
    let mut candidates = vec![full.clone()];

    if let Some(dot_pos) = full.find('.') {
        let suffix_start = full.find(|c: char| c.is_alphabetic()).unwrap_or(full.len());
        let suffix = &full[suffix_start..];
        let integer_part = &full[..dot_pos];
        let decimal_part = &full[dot_pos + 1..suffix_start];

        // Progressively truncate decimals from the right
        for keep in (0..decimal_part.len()).rev() {
            let truncated_digits = &decimal_part[keep..];
            let has_nonzero = truncated_digits.chars().any(|c| c != '0');

            if keep == 0 {
                // No decimals left - just integer + suffix
                let int_val: u64 = integer_part.parse().unwrap_or(0);
                let rounded = if has_nonzero { int_val + 1 } else { int_val };
                candidates.push(format!("{}{}", rounded, suffix));
            } else {
                let mut kept_chars: Vec<u8> = decimal_part[..keep].bytes().collect();
                if has_nonzero {
                    // Round up: increment last kept digit, propagate carry
                    let mut carry = true;
                    for d in kept_chars.iter_mut().rev() {
                        if carry {
                            if *d == b'9' {
                                *d = b'0';
                            } else {
                                *d += 1;
                                carry = false;
                            }
                        }
                    }
                    if carry {
                        // Carry overflowed all decimal digits (e.g. .99 → 1.0)
                        // Increment integer part instead
                        let int_val: u64 = integer_part.parse().unwrap_or(0);
                        candidates.push(format!("{}{}", int_val + 1, suffix));
                        continue;
                    }
                }
                let kept_str: String = kept_chars.iter().map(|&b| b as char).collect();
                candidates.push(format!("{}.{}{}", integer_part, kept_str, suffix));
            }
        }
    }

    candidates
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
/// - Bytes 0-8 contain two empty DEFLATE blocks (non-final + final) for None filter
/// - Bytes 9+ are zero (black)
/// - All rows use None filter (0x00)
fn render_filename_label(name: &[u8], row_width: usize, fonts: &FontSelection, header_row: &[u8], is_terminator: bool, bridge_bytes: usize, file_size: usize, bytes_per_pixel: usize) -> Vec<u8> {
    let font = fonts.name_font;
    let size_font = fonts.size_font;
    let label_rows = label_rows_for_font(fonts);
    let mut result = Vec::with_capacity(label_rows * row_width);

    // Lay out name text with canvas-based kerning
    let name_str: String = name.iter().map(|&b| b as char).collect();
    let (canvas, char_positions, actual_width) = font.layout_text(&name_str);

    // Re-fetch lookups for rendering (layout_text only returns positions)
    let lookups: Vec<Option<fonts::GlyphLookup<'_>>> = name_str.chars()
        .map(|c| font.get_glyph(c))
        .collect();

    // Calculate starting x position (in pixel coordinates)
    // Default: 4px margin from left edge
    // If text overflows: right-align, but always leave 1px at right edge for halo
    const MARGIN: usize = 4;
    const RIGHT_PADDING: usize = 1; // Always leave 1px at right for halo
    let min_gap = font.max_ink_width() + 3; // Dynamic gap: one full character width + 3px padding
    let pixel_width = row_width / bytes_per_pixel.max(1);
    let usable_width = pixel_width - RIGHT_PADDING;

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
        pixel_width // Filename overflows, no room for size
    };

    // Try progressively shorter file size candidates until one fits
    let candidates = format_file_size_candidates(file_size);
    let mut render_size = false;
    let mut size_str = String::new();
    let mut size_char_positions: Vec<(usize, i32)> = Vec::new();
    let mut size_start_x: i32 = 0;

    for candidate in &candidates {
        let (_canvas, positions, width) = size_font.layout_text(candidate);
        let start = usable_width as i32 - width as i32 - MARGIN as i32;
        if start >= 0 && start as usize >= filename_end_x.saturating_add(min_gap) {
            render_size = true;
            size_str = candidate.clone();
            size_char_positions = positions;
            size_start_x = start;
            break;
        }
    }

    let size_lookups: Vec<Option<fonts::GlyphLookup<'_>>> = size_str.chars()
        .map(|c| size_font.get_glyph(c))
        .collect();

    // First, render text to a temporary bitmap to know where pixels are
    // Bitmap is in pixel coordinates (pixel_width wide)
    let mut text_bitmap: Vec<Vec<bool>> = vec![vec![false; pixel_width]; label_rows];

    // Padding row above (row 0) has no text
    // Text area spans rows 1..=max_height, with fonts bottom-aligned
    // Padding row below is max_height + 1
    let max_height = fonts.max_height();
    let name_y_offset = max_height - font.height; // bottom-align name font
    let size_y_offset = max_height - size_font.height; // bottom-align size font

    // Render filename
    for text_row in 0..font.height {
        let result_row = text_row + 1 + name_y_offset; // +1 for padding above, offset for alignment
        for &(char_idx, char_x) in &char_positions {
            if let Some(lookup) = &lookups[char_idx] {
                let glyph = lookup.glyph;
                if text_row < glyph.len() {
                    for (px, &pixel_on) in glyph[text_row].iter().enumerate() {
                        let x = (start_x + char_x + px as i32) as usize;
                        if x < pixel_width && pixel_on {
                            text_bitmap[result_row][x] = true;
                        }
                    }
                }
            }
        }
    }

    // Render file size (right-aligned) if there's enough space
    if render_size {
        for text_row in 0..size_font.height {
            let result_row = text_row + 1 + size_y_offset;
            for &(char_idx, char_x) in &size_char_positions {
                if let Some(lookup) = &size_lookups[char_idx] {
                    let glyph = lookup.glyph;
                    if text_row < glyph.len() {
                        for (px, &pixel_on) in glyph[text_row].iter().enumerate() {
                            let x = (size_start_x + char_x + px as i32) as usize;
                            if x < pixel_width && pixel_on {
                                text_bitmap[result_row][x] = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Header "stretches" up from bottom row, but stops:
    // - when it hits the text halo, OR
    // - 2 pixels from the top of the label space (whichever comes first)
    let stretch_top_limit = 2; // don't stretch into the top 2 rows
    let meaningful_header_len = (30 + name.len()).min(header_row.len());

    // Track per-pixel-column whether the stretch is still active
    let mut stretch_active: Vec<bool> = vec![true; pixel_width];
    // rows_data is in bytes (row_width per row), initialized to opaque black for RGBA.
    // All rows use None filter (0x00), so pre-filtered data = raw pixel values.
    // The terminator row (first row, if is_terminator) is overwritten below.
    let mut rows_data: Vec<Vec<u8>> = if bytes_per_pixel > 1 {
        let mut rows = Vec::with_capacity(label_rows);
        for _ in 0..label_rows {
            let mut row = Vec::with_capacity(row_width);
            for _px in 0..(row_width / bytes_per_pixel) {
                for c in 0..bytes_per_pixel {
                    row.push(if c == bytes_per_pixel - 1 { 0xFF } else { 0x00 });
                }
            }
            rows.push(row);
        }
        rows
    } else {
        vec![vec![0u8; row_width]; label_rows]
    };

    // Process from bottom to top (header stretches upward)
    // Halo detection works in pixel coordinates; header copy works in byte coordinates
    for row_idx in (0..label_rows).rev() {
        // Stop stretching once we reach the top limit
        if row_idx < stretch_top_limit {
            break;
        }

        #[expect(clippy::needless_range_loop)]
        for px in 0..pixel_width {
            if !stretch_active[px] {
                continue; // Already blocked in this column
            }

            // Check if this pixel position is in the halo (adjacent to text)
            let mut near_text = false;
            'halo: for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let ny = row_idx as i32 + dy;
                    let nx = px as i32 + dx;
                    if ny >= 0 && (ny as usize) < label_rows && nx >= 0 && (nx as usize) < pixel_width
                        && text_bitmap[ny as usize][nx as usize] {
                            near_text = true;
                            break 'halo;
                        }
                }
            }

            if near_text {
                // Block this column from stretching further up
                stretch_active[px] = false;
            } else {
                // Stretch continues - copy header bytes as-is (raw data, not pixel-aware)
                let byte_start = px * bytes_per_pixel;
                for b in 0..bytes_per_pixel {
                    let byte_idx = byte_start + b;
                    if byte_idx < meaningful_header_len {
                        rows_data[row_idx][byte_idx] = header_row[byte_idx];
                    }
                }
            }
        }
    }

    // Draw text pixels on top (white = all channels 0xFF)
    for row_idx in 0..label_rows {
        #[expect(clippy::needless_range_loop)]
        for px in 0..pixel_width {
            if text_bitmap[row_idx][px] {
                let byte_start = px * bytes_per_pixel;
                for b in 0..bytes_per_pixel {
                    rows_data[row_idx][byte_start + b] = 0xFF;
                }
            }
        }
    }

    // If this is a terminator label, overwrite the first row with terminator bytes.
    // Uses None filter (0x00) — the double-block layout is displayed as raw bytes.
    if is_terminator {
        let terminator = create_terminator_row(row_width, bridge_bytes);
        rows_data[0] = terminator;
    }

    // Output all rows
    for row in &rows_data {
        result.extend_from_slice(row);
    }

    result
}

/// Estimate total data size for width calculation.
fn estimate_total_size(files: &[(&[u8], &[u8])], font: Option<&FontSelection>) -> usize {
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

/// Round up so that:
/// 1. The data portion (row_width - 4) is a multiple of DATA_ALIGNMENT (64), and
/// 2. row_width is a multiple of bytes_per_pixel.
///
/// Both constraints are needed: (1) ensures each row contains whole 64-byte deflate
/// blocks, and (2) ensures the PNG decoder computes the correct bytes_per_scanline.
/// Without (2), `effective_width = row_width / bytes_per_pixel` truncates via integer
/// division, and the decoder expects fewer bytes per scanline than actually present,
/// causing data bytes to be misinterpreted as filter bytes ("Unknown filter method N").
///
/// The combined period is LCM(DATA_ALIGNMENT, bytes_per_pixel). We find the offset
/// within that period where both constraints hold, then align to it.
fn align_to_row_width(value: usize, bytes_per_pixel: usize) -> usize {
    // Period for the combined constraints
    let period = lcm(DATA_ALIGNMENT, bytes_per_pixel);
    // Find the offset within `period` where row_width % bpp == 0 and (row_width - 4) % 64 == 0.
    // Since period = LCM(64, bpp), any row_width = offset + k*period satisfying both
    // constraints at offset will satisfy them for all k.
    // We need: offset % bpp == 0 AND (offset - 4) % 64 == 0, i.e. offset ≡ 4 (mod 64).
    // Search within one period (always small: at most LCM(64, 4) = 64).
    let offset = (0..=period)
        .find(|&o| o % bytes_per_pixel == 0 && o >= DEFLATE_HEADER_OVERHEAD && (o - DEFLATE_HEADER_OVERHEAD).is_multiple_of(DATA_ALIGNMENT))
        .expect("no valid alignment offset found");

    // Find smallest row_width >= value matching: row_width = offset + k * period
    
    if value <= offset {
        offset
    } else {
        let k = (value - offset).div_ceil(period);
        offset + k * period
    }
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn lcm(a: usize, b: usize) -> usize {
    a / gcd(a, b) * b
}

/// Calculate optimal row width for approximately square images.
/// Returns width such that height >= width (portrait/square orientation).
/// The data portion (row_width - 4) is always a multiple of 64 bytes.
///
/// The `min_width` parameter ensures the row is wide enough to fit ZIP headers
/// without spanning multiple rows (which would corrupt them with filter bytes).
fn calculate_row_width(total_data_estimate: usize, min_width: usize, bytes_per_pixel: usize) -> usize {
    // For height >= width, we need: total_data / width >= width
    // Therefore: width <= sqrt(total_data)
    // Using floor ensures height >= width
    let ideal = (total_data_estimate as f64).sqrt();
    let width = ideal.floor() as usize;

    // Clamp to minimum safe width and align data portion to LCM(DATA_ALIGNMENT, bytes_per_pixel)
    align_to_row_width(width.max(min_width), bytes_per_pixel)
}

/// Calculate minimum row width needed for a set of files.
/// Ensures ZIP local file headers (30 bytes + filename) fit in one row.
/// Returns a value where (row_width - 4) is a multiple of 64.
fn min_row_width_for_files(files: &[(&[u8], &[u8])], bytes_per_pixel: usize) -> usize {
    let longest_filename = files.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    // Header is 30 bytes + filename; add 4 bytes padding for safety
    let min_for_headers = 30 + longest_filename + 4;
    align_to_row_width(min_for_headers.max(BASE_MIN_ROW_WIDTH), bytes_per_pixel)
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

    // Calculate bytes per pixel for row alignment (for 8-bit depth, this is samples_per_pixel)
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let bytes_per_pixel = bits_per_pixel.div_ceil(8);

    // Calculate minimum row width based on filename lengths
    let min_width = min_row_width_for_files(files, bytes_per_pixel);

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
    let estimated_size = estimate_total_size(files, font.as_ref());
    let initial_width = calculate_row_width(estimated_size, min_width, bytes_per_pixel);
    let (initial_data, _) = build_aligned_data(files, initial_width, font.as_ref(), bytes_per_pixel);

    // Pass 2: Calculate optimal width from actual size
    let actual_size = initial_data.len();
    let mut row_width = calculate_row_width(actual_size, min_width, bytes_per_pixel);

    // Optimization: If all files would fit in one content row each, use narrower width.
    // This matters for many small files where the "square" heuristic wastes space.
    let max_body_len = files.iter().map(|(_, body)| body.len()).max().unwrap_or(0);
    // For one content row: body must fit in (row_width - 4) data area
    // So min width = max_body + 4, then align to LCM(DATA_ALIGNMENT, bytes_per_pixel)
    let min_width_for_single_row = align_to_row_width((max_body_len + DEFLATE_HEADER_OVERHEAD).max(min_width), bytes_per_pixel);

    if min_width_for_single_row < row_width {
        // Verify: at this width, do all files fit in one content row?
        let data_per_row = min_width_for_single_row - DEFLATE_HEADER_OVERHEAD;
        let all_fit = files.iter().all(|(_, body)| body.len() <= data_per_row);
        if all_fit {
            row_width = min_width_for_single_row;
        }
    }

    // Build with the (possibly optimized) row_width
    let (pixel_data, entry_infos) = build_aligned_data(files, row_width, font.as_ref(), bytes_per_pixel);

    // Step 2: Build file entries with correct offsets (need this before CD+EOCD)
    // Calculate offset where filtered pixel data starts in the file
    // PNG sig (8) + IHDR chunk (25) + PLTE if any + IDAT header (8) + zlib (2) + deflate (5)
    let plte_size = palette.map(|p| 4 + 4 + p.len() + 4).unwrap_or(0);
    let data_offset = 8 + 25 + plte_size + 8 + 2 + 5;

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

    // Step 3: Build CD+EOCD as pixel rows inside IDAT.
    //
    // The CD+EOCD bytes become the last rows of pixel data. Zero-valued fields
    // in the CD and EOCD structures naturally provide valid PNG None filter bytes
    // (0x00) at row boundaries. Extra field padding is inserted when needed to
    // align row boundaries onto these zero bytes.
    //
    // Layout (decompressed):
    //   filtered_pixels (pixel rows with filter bytes)
    //   + aligned CD+EOCD (also with filter-compatible row boundaries)
    //
    // From ZIP's perspective the CD+EOCD bytes are contiguous (extra fields are
    // valid ZIP). From PNG's perspective every row starts with 0x00 (None filter).

    let filtered_row_size = row_width + 1;

    // Pad pixel_data to full rows
    let pixel_height = if pixel_data.is_empty() { 1 } else { pixel_data.len().div_ceil(row_width) };
    let mut padded_pixels = pixel_data.clone();
    resize_with_opaque_padding(&mut padded_pixels, pixel_height * row_width, bytes_per_pixel);

    // Filter the pixel data
    let mut filtered_pixels = add_filter_bytes(&padded_pixels, row_width);

    // Ensure the CD+EOCD won't be split by an IDAT stored deflate block boundary.
    // The 5-byte stored block headers appear in the file every IDAT_BLOCK_SIZE bytes
    // of decompressed data. ZIP parsers see the raw file bytes, so any block header
    // landing inside the CD would corrupt it.
    //
    // Strategy: estimate CD+EOCD size, check if it fits in the current IDAT block.
    // If not, pad filtered_pixels with zero rows to reach the next block boundary.
    // Estimate total size of CD+EOCD + trailing gradient to check IDAT boundary
    let trailing_gradient_size = if bytes_per_pixel > 1 {
        filtered_row_size  // 1 row for RGBA
    } else {
        256_usize.div_ceil(row_width) * filtered_row_size  // gradient rows for indexed
    };
    let cd_eocd_estimate = file_entries.iter().map(|e| 46 + e.name.len()).sum::<usize>()
        + 22  // EOCD
        + file_entries.len() * filtered_row_size  // generous padding estimate
        + filtered_row_size  // final row padding
        + trailing_gradient_size;  // trailing gradient after CD
    let space_in_current_block = IDAT_BLOCK_SIZE - (filtered_pixels.len() % IDAT_BLOCK_SIZE);
    if cd_eocd_estimate > space_in_current_block {
        // Pad to next IDAT block boundary with whole filtered rows
        let pad_needed = space_in_current_block;
        let pad_rows = pad_needed.div_ceil(filtered_row_size);
        let old_len = filtered_pixels.len();
        filtered_pixels.resize(old_len + pad_rows * filtered_row_size, 0);
    }

    // File prefix size: bytes before the decompressed stream in the file.
    // PNG_sig(8) + IHDR(25) + PLTE(if any) + IDAT_chunk_header(8) + zlib(2)
    let file_prefix_size = 8 + 25 + plte_size + 8 + 2;

    // Build aligned CD+EOCD
    let cd_eocd = build_aligned_cd_eocd(
        &file_entries,
        filtered_row_size,
        filtered_pixels.len(),
        file_prefix_size,
    );

    // Concatenate filtered pixels + CD+EOCD + trailing gradient
    let mut all_filtered = filtered_pixels;
    all_filtered.extend_from_slice(&cd_eocd);

    // Append trailing gradient after CD+EOCD (with filter bytes)
    let trailing_gradient = build_trailing_gradient(row_width, bytes_per_pixel);
    for row in trailing_gradient.chunks(row_width) {
        all_filtered.push(0x00); // None filter byte
        all_filtered.extend_from_slice(row);
    }

    // Total height from combined decompressed size
    let height = all_filtered.len() / filtered_row_size;
    assert_eq!(all_filtered.len() % filtered_row_size, 0,
        "Total filtered data ({}) must be exact multiple of filtered_row_size ({})",
        all_filtered.len(), filtered_row_size);

    // Calculate effective pixel width based on bit depth
    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let effective_width = (row_width * 8) / bits_per_pixel;

    // Step 4: Build PNG with CD+EOCD as pixel rows inside IDAT
    let mut output = output_buffer();
    let _ = write_png_header(&mut output, effective_width as u32, height as u32, bit_depth, color_mode);

    if let Some(p) = palette {
        let plte_data = OutputBuffer::without_tag(p);
        let _ = write_png_chunk(&mut output, b"PLTE", &plte_data);
    }

    // Write IDAT — all data is in filtered pixel rows, no trailing block needed
    write_idat_stored(&mut output, &all_filtered);

    let _ = write_png_footer(&mut output);

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
    let target_row = next_boundary.div_ceil(filtered_row_size);

    // Return the data position for the start of that row
    target_row * row_width
}

/// Calculate the total size a file will occupy (label + header + content + terminator).
/// Note: When labels are enabled, the terminator may merge with the next file's label,
/// but we count it conservatively for bin packing purposes.
fn calculate_file_size(name: &[u8], body: &[u8], row_width: usize, font: Option<&FontSelection>) -> usize {
    let data_per_block = row_width - 4;
    let header_size = 30 + name.len();
    let bytes_for_alignment = header_size % row_width;
    let extra_len = if bytes_for_alignment == 0 { 0 } else { row_width - bytes_for_alignment };
    let num_blocks = if body.is_empty() { 1 } else { body.len().div_ceil(data_per_block) };
    let file_data_size = header_size + extra_len + num_blocks * row_width;
    let label_size = font.map(|f| label_rows_for_font(f) * row_width).unwrap_or(0);
    // Add terminator row only if neither internal nor embedded terminator is used
    let uses_embedded = embeds_terminator_in_padding(body.len(), row_width);
    let terminator_size = if uses_embedded { 0 } else { row_width };
    label_size + file_data_size + terminator_size
}

/// Resize `data` to `new_len`, filling new bytes with opaque black pixels.
/// For indexed mode (bytes_per_pixel == 1), fills with 0x00.
/// For RGB/RGBA (bytes_per_pixel > 1), fills with the pattern [0x00, ..., 0xFF]
/// per pixel so that the alpha channel is opaque (0xFF).
/// Resize data with opaque black padding for RGBA, or zero padding for indexed.
///
/// The data buffer holds pre-filtered pixel data (filter is applied separately by
/// `add_filter_bytes`). Padding rows always use None filter (0x00), so the
/// pre-filtered data equals the raw pixel values. For RGBA, we fill with
/// `[0x00, 0x00, 0x00, 0xFF]` per pixel (opaque black). For indexed (bpp=1),
/// we fill with 0x00 (palette index 0 = black in the diagnostic palette).
///
/// Note: terminator rows are handled separately by `create_terminator_row`,
/// NOT by this function.
fn resize_with_opaque_padding(data: &mut Vec<u8>, new_len: usize, bytes_per_pixel: usize) {
    if new_len <= data.len() {
        data.truncate(new_len);
        return;
    }
    if bytes_per_pixel <= 1 {
        data.resize(new_len, 0);
        return;
    }
    let old_len = data.len();
    data.reserve(new_len - old_len);
    // Fill with opaque black: [0x00, ..., 0x00, 0xFF] per pixel.
    // Since padding always starts at a row boundary and row_width is a multiple
    // of bytes_per_pixel, the absolute byte index gives the correct channel position.
    for i in old_len..new_len {
        if i % bytes_per_pixel == bytes_per_pixel - 1 {
            data.push(0xFF); // alpha = opaque
        } else {
            data.push(0x00);
        }
    }
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
#[expect(clippy::type_complexity)]
fn build_aligned_data(
    files: &[(&[u8], &[u8])],
    row_width: usize,
    font: Option<&FontSelection>,
    bytes_per_pixel: usize,
) -> (Vec<u8>, Vec<(Vec<u8>, Vec<u8>, usize, usize)>) {
    let data_per_block = row_width - 4;
    let filtered_row_size = row_width + 1;

    if files.is_empty() {
        return (Vec::new(), Vec::new());
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
    let bucket_spacing = calculate_bucket_spacing(&bucket_assignments, &file_sizes, row_width, font.is_some());

    // Phase 4: Place files with pre-calculated spacing
    // Each file's DEFLATE stream needs a terminator row (BFINAL=1).
    // - If the next file has a label with no spacing gap, the label's first row is the terminator
    // - Otherwise, add an explicit terminator row after the content
    let mut data = Vec::new();
    let mut entries = Vec::new();
    // All rows use None filter (0x00). No terminator row tracking needed.

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
    let mut pending_bridge_bytes: usize = 0;

    // Insert reference color rows at the very start of the image.
    if bytes_per_pixel > 1 {
        // RGBA mode: single row cycling through 6 reference colors as RGBA pixels
        // transparent, black, white, red, green, blue
        const RGBA_REF_COLORS: [[u8; 4]; 6] = [
            [0, 0, 0, 0],       // transparent
            [0, 0, 0, 255],     // black
            [255, 255, 255, 255], // white
            [255, 0, 0, 255],   // red
            [0, 255, 0, 255],   // green
            [0, 0, 255, 255],   // blue
        ];
        data.reserve(row_width);
        let pixels_per_row = row_width / bytes_per_pixel;
        for px in 0..pixels_per_row {
            let color = &RGBA_REF_COLORS[px % RGBA_REF_COLORS.len()];
            data.extend_from_slice(color);
        }
    } else {
        // Indexed mode: cycling gradient (0→255→255→0→...) spanning ≥256 bytes,
        // letting readers map pixel colors back to byte values.
        let reverse_color_map_rows = 256_usize.div_ceil(row_width);
        let mut palette_counter: usize = 0;
        let color_map_bytes = reverse_color_map_rows * row_width;
        data.reserve(color_map_bytes);
        for _ in 0..color_map_bytes {
            data.push(cycling_palette_index(&mut palette_counter));
        }
    }


    let rows_per_bucket = IDAT_BLOCK_SIZE / filtered_row_size;
    let max_file_rows = rows_per_bucket;

    for (order_idx, &(file_idx, spacing_rows)) in file_order_with_spacing.iter().enumerate() {
        let has_spacing_gap = spacing_rows > 0;

        // Check if this file with label would exceed one IDAT block.
        // If so, skip the label for this file to avoid crossing an IDAT boundary
        // (which would insert a 5-byte deflate block header mid-ZIP-stream).
        let (name, body) = &files[file_idx];
        let file_size_with_label = file_sizes[file_idx];
        let file_rows_with_label = file_size_with_label.div_ceil(row_width);
        // If file+label exceeds one block, split: place label before the IDAT
        // boundary and file after. The 5-byte deflate block header between them
        // is invisible in pixel data — only ZIP data must be contiguous.
        let (has_label, split_label) = if font.is_some() && file_rows_with_label > max_file_rows {
            let file_size_without_label = calculate_file_size(name, body, row_width, None);
            let file_rows_without_label = file_size_without_label.div_ceil(row_width);
            assert!(
                file_rows_without_label <= max_file_rows,
                "File {:?} ({} rows) exceeds single IDAT block capacity ({} rows) even without label",
                String::from_utf8_lossy(name), file_rows_without_label, max_file_rows
            );
            (true, true)
        } else {
            (font.is_some(), false)
        };
        let actual_file_size = if has_label && !split_label {
            file_size_with_label
        } else {
            calculate_file_size(name, body, row_width, None)
        };

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
            let start_filtered = data_to_filtered_pos(pos_after_align, row_width);
            let end_filtered = data_to_filtered_pos(pos_after_align + actual_file_size, row_width);
            if crosses_idat_boundary(start_filtered, end_filtered) {
                // Boundary crossing will happen - label can't serve as terminator
                label_is_terminator = false;
            }
        }

        // If previous file needs terminator and we can't use this label, add explicit one
        if pending_terminator && !label_is_terminator {
            let terminator = create_terminator_row(row_width, pending_bridge_bytes);
            data.extend_from_slice(&terminator);
        }
        // Note: pending_terminator will be set at the end of this iteration

        // Add spacing BEFORE this file (after any terminator)
        if spacing_rows > 0 {
            // Align to row first, then add spacing rows (zero-filled)
            let padding = (row_width - (data.len() % row_width)) % row_width;
            let new_len = data.len() + padding + spacing_rows * row_width;
            resize_with_opaque_padding(&mut data, new_len, bytes_per_pixel);
        }

        // Align to row boundary
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        let new_len = data.len() + padding_to_row;
        resize_with_opaque_padding(&mut data, new_len, bytes_per_pixel);

        // When splitting, place label BEFORE the boundary so it appears visually
        // adjacent to the file. The IDAT boundary (5-byte deflate header) between
        // label and file is invisible in the pixel data.
        if split_label && has_label {
            let f = font.unwrap();
            // We need the header bytes for label rendering; pre-compute them here.
            let header_size = 30 + name.len();
            let bytes_used = header_size % row_width;
            let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };
            let compressed_size = calculate_compressed_size(body.len(), row_width);
            let crc = crc32(body);
            let header_bytes = build_local_header(name, body.len(), compressed_size, crc, extra_len);
            let label_bridge = if label_is_terminator { pending_bridge_bytes } else { 0 };
            let label = render_filename_label(name, row_width, f, &header_bytes, label_is_terminator, label_bridge, body.len(), bytes_per_pixel);
            data.extend_from_slice(&label);
        }

        // Check if this file would cross an IDAT boundary
        let start_filtered = data_to_filtered_pos(data.len(), row_width);
        let end_filtered = data_to_filtered_pos(data.len() + actual_file_size, row_width);
        if crosses_idat_boundary(start_filtered, end_filtered) {
            // Pad to next boundary-aligned position
            let boundary_target = next_boundary_aligned_pos(data.len(), row_width);
            resize_with_opaque_padding(&mut data, boundary_target, bytes_per_pixel);
        }

        // Place the file
        // Calculate header and extra field sizes
        let header_size = 30 + name.len();
        let bytes_used = header_size % row_width;
        let extra_len = if bytes_used == 0 { 0 } else { row_width - bytes_used };

        // Calculate number of deflate blocks and compressed size
        let compressed_size = calculate_compressed_size(body.len(), row_width);

        // Pre-compute the header bytes
        let crc = crc32(body);
        let header_bytes = build_local_header(name, body.len(), compressed_size, crc, extra_len);

        // Insert filename label if not already placed before the boundary (split case)
        if has_label && !split_label {
            let f = font.unwrap();
            let label_bridge = if label_is_terminator { pending_bridge_bytes } else { 0 };
            let label = render_filename_label(name, row_width, f, &header_bytes, label_is_terminator, label_bridge, body.len(), bytes_per_pixel);
            data.extend_from_slice(&label);
        }

        let entry_start = data.len();

        entries.push((name.to_vec(), body.to_vec(), entry_start, compressed_size));

        // Write the header and content
        data.extend_from_slice(&header_bytes);
        let mut embedded_terminator = false;
        let mut file_bridge_bytes: usize = 0;
        let deflate_content = encode_as_deflate_blocks(body, row_width, &mut embedded_terminator, &mut file_bridge_bytes);
        data.extend_from_slice(&deflate_content);

        if embedded_terminator {
            // Terminator is embedded in the padding area of the last content row.
            // No separate terminator row needed.
            pending_terminator = false;
            pending_bridge_bytes = 0;
        } else {
            // This file needs a separate terminator row
            pending_terminator = true;
            pending_bridge_bytes = file_bridge_bytes;
        }
    }

    // Handle terminator for the very last file
    if pending_terminator {
        let terminator = create_terminator_row(row_width, pending_bridge_bytes);
        data.extend_from_slice(&terminator);
    }

    // Append gap + 0xFF separator row before the CD data.
    {
        let trailing_gap_rows: usize = if font.is_some() { 2 } else { 1 };

        // Align to row boundary first
        let padding_to_row = (row_width - (data.len() % row_width)) % row_width;
        let new_len = data.len() + padding_to_row;
        resize_with_opaque_padding(&mut data, new_len, bytes_per_pixel);

        // Add gap rows before separator
        let new_len = data.len() + trailing_gap_rows * row_width;
        resize_with_opaque_padding(&mut data, new_len, bytes_per_pixel);

        // Single row of all 0xFF (max palette index or white pixels)
        data.extend_from_slice(&vec![0xFF; row_width]);
    }

    (data, entries)
}

/// Build the trailing mirrored reference gradient rows (unfiltered pixel data).
/// For indexed mode: reversed cycling gradient spanning ≥256 bytes.
/// For RGBA mode: single row with reversed 6-color cycle.
fn build_trailing_gradient(row_width: usize, bytes_per_pixel: usize) -> Vec<u8> {
    if bytes_per_pixel > 1 {
        const RGBA_REF_COLORS: [[u8; 4]; 6] = [
            [0, 0, 0, 0],       // transparent
            [0, 0, 0, 255],     // black
            [255, 255, 255, 255], // white
            [255, 0, 0, 255],   // red
            [0, 255, 0, 255],   // green
            [0, 0, 255, 255],   // blue
        ];
        let pixels_per_row = row_width / bytes_per_pixel;
        let mut forward: Vec<[u8; 4]> = Vec::with_capacity(pixels_per_row);
        for px in 0..pixels_per_row {
            forward.push(RGBA_REF_COLORS[px % RGBA_REF_COLORS.len()]);
        }
        forward.reverse();
        let mut result = Vec::with_capacity(row_width);
        for color in &forward {
            result.extend_from_slice(color);
        }
        result
    } else {
        let trailing_color_map_rows = 256_usize.div_ceil(row_width);
        let color_map_bytes = trailing_color_map_rows * row_width;
        let mut forward: Vec<u8> = Vec::with_capacity(color_map_bytes);
        let mut palette_counter: usize = 0;
        for _ in 0..color_map_bytes {
            forward.push(cycling_palette_index(&mut palette_counter));
        }
        forward.reverse();
        forward
    }
}

/// Returns a cycling palette index: 0,1,2,...,255,255,254,...,1,0,0,1,...
/// The counter is incremented for each call.
fn cycling_palette_index(counter: &mut usize) -> u8 {
    let i = *counter % 512;
    *counter += 1;
    if i <= 255 { i as u8 } else { (511 - i) as u8 }
}

/// Calculate spacing for each bucket.
/// Full buckets get even spacing with half-weight edges: [0.5, 1, 1, ..., 1, 0.5].
/// The first bucket includes a leading gap (space between the reverse color map and
/// the first file), so it also uses half-weight edges. The first bucket's capacity
/// is reduced by the preamble rows (1 zero row + reverse color map rows).
/// Last bucket gets no spacing (content packed tight).
fn calculate_bucket_spacing(
    bucket_assignments: &[Vec<usize>],
    file_sizes: &[usize],
    row_width: usize,
    has_labels: bool,
) -> Vec<Vec<usize>> {
    let filtered_row_size = row_width + 1;
    let rows_per_bucket = IDAT_BLOCK_SIZE / filtered_row_size;
    let bucket_capacity_bytes = rows_per_bucket * row_width;

    // Preamble: reference color map rows at the top of the image.
    // The gap after the color map is part of normal spacing distribution (not preamble).
    let reverse_color_map_rows = 256_usize.div_ceil(row_width);
    let preamble_bytes = reverse_color_map_rows * row_width;

    let num_buckets = bucket_assignments.len();
    let mut result = Vec::with_capacity(num_buckets);

    for (bucket_idx, file_indices) in bucket_assignments.iter().enumerate() {
        let is_first_bucket = bucket_idx == 0;
        let is_last_bucket = bucket_idx == num_buckets - 1;
        let num_files = file_indices.len();

        if num_files == 0 {
            result.push(vec![]);
            continue;
        }

        // When labels are present, no minimum gap needed (labels provide visual separation).
        // Without labels, enforce a minimum of 1 row of spacing before each file.
        let min_per_file: usize = if has_labels { 0 } else { 1 };
        let min_spacing_rows = num_files * min_per_file;

        // Last bucket: no extra distribution, just the minimum.
        if is_last_bucket {
            result.push(vec![min_per_file; num_files]);
            continue;
        }

        // Calculate total content bytes
        let total_content: usize = file_indices.iter()
            .map(|&idx| file_sizes[idx])
            .sum();

        // First bucket has reduced capacity due to the preamble (reverse color map rows)
        let effective_capacity = if is_first_bucket {
            bucket_capacity_bytes.saturating_sub(preamble_bytes)
        } else {
            bucket_capacity_bytes
        };

        // Slack after reserving minimum spacing for each file
        let reserved_bytes = min_spacing_rows * row_width;
        let slack_bytes = effective_capacity.saturating_sub(total_content + reserved_bytes);
        let extra_slack_rows = slack_bytes / row_width;

        if extra_slack_rows == 0 {
            // No room beyond the minimums
            result.push(vec![min_per_file; num_files]);
            continue;
        }

        // Distribute extra slack with half-weight edges: [0.5, 1, 1, ..., 1, 0.5]
        let first_weight = 0.5;
        let total_weight = first_weight + (num_files - 1) as f64 + 0.5;
        let rows_per_unit = extra_slack_rows as f64 / total_weight;

        let mut spacing = Vec::with_capacity(num_files);
        let mut allocated = 0usize;

        for i in 0..num_files {
            let weight = if i == 0 { first_weight } else { 1.0 };
            let rows = min_per_file + (weight * rows_per_unit).floor() as usize;
            spacing.push(rows);
            allocated += rows - min_per_file; // track only the extra part
        }

        // Distribute remainder of extra slack
        let mut remaining = extra_slack_rows.saturating_sub(allocated);
        for s in &mut spacing {
            if remaining == 0 { break; }
            *s += 1;
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
fn encode_as_deflate_blocks(body: &[u8], row_width: usize, embedded_terminator: &mut bool, bridge_bytes: &mut usize) -> Vec<u8> {
    let data_per_block = row_width - 4;
    let mut result = Vec::new();

    *embedded_terminator = false;
    *bridge_bytes = 0;

    if body.is_empty() {
        // Empty file: single stored block with 0 length
        result.extend_from_slice(&0_u16.to_le_bytes());
        result.extend_from_slice(&0xFFFF_u16.to_le_bytes());
        // Fill padding with empty stored blocks
        let padding_needed = row_width - 4;
        fill_padding_with_empty_blocks(&mut result, padding_needed, embedded_terminator, bridge_bytes);
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
                fill_padding_with_empty_blocks(&mut result, padding_needed, embedded_terminator, bridge_bytes);
            } else {
                // Non-last rows with partial data - shouldn't happen normally
                result.resize(result.len() + padding_needed, 0);
            }
        }
    }

    result
}

/// Fill padding with valid DEFLATE blocks.
///
/// When padding is divisible by 5: embed BFINAL=1 terminator in padding.
/// When padding % 5 != 0: fill with empty non-final blocks + bridge bytes
/// that start a stored block spanning into the next (terminator) row.
/// The `bridge_bytes` output indicates how many bytes bridge into the next row.
///
/// Bridge byte patterns for remainder R = padding % 5:
///   R=0: (all padding is complete blocks, no bridge needed)
///   R=1: `00`           — starts a stored block header
///   R=2: `00 00`        — header + LEN_lo=0
///   R=3: `02 00 00`     — fixed Huffman block + header + LEN_lo=0
///   R=4: `02 08 00 00`  — two fixed Huffman blocks + header + LEN_lo=0
fn fill_padding_with_empty_blocks(result: &mut Vec<u8>, padding_needed: usize, embedded_terminator: &mut bool, bridge_bytes: &mut usize) {
    if padding_needed == 0 {
        return;
    }

    *bridge_bytes = 0;

    if padding_needed.is_multiple_of(5) {
        // Embed BFINAL=1 terminator in padding, fill rest with zeros.
        // compressed_size will be shortened to exclude the trailing zeros.
        result.push(0x01); // BFINAL=1, BTYPE=00 (stored)
        result.extend_from_slice(&0_u16.to_le_bytes());      // LEN=0
        result.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // NLEN=0xFFFF
        if padding_needed > 5 {
            result.resize(result.len() + padding_needed - 5, 0); // black pixels (dead space)
        }
        *embedded_terminator = true;
    } else {
        let remainder = padding_needed % 5;
        let full_blocks = padding_needed / 5;

        // Fill complete 5-byte empty non-final stored blocks
        for _ in 0..full_blocks {
            result.push(0x00); // BFINAL=0, BTYPE=00
            result.extend_from_slice(&0_u16.to_le_bytes());      // LEN=0
            result.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // NLEN=0xFFFF
        }

        // Fill bridge bytes for the remainder
        match remainder {
            1 => {
                // Start a stored block that spans into the terminator row
                result.push(0x00); // BFINAL=0, BTYPE=00 (stored block header)
            }
            2 => {
                // Header + LEN_lo=0
                result.push(0x00); // BFINAL=0, BTYPE=00
                result.push(0x00); // LEN_lo=0
            }
            3 => {
                // Non-final fixed Huffman block (2 bytes) + stored block start (1 byte)
                result.push(0x02); // BFINAL=0, BTYPE=01 (fixed Huffman)
                result.push(0x00); // end-of-block bits + next block header bits
                result.push(0x00); // LEN_lo=0 (after stored block skip-to-byte)
            }
            4 => {
                // Two fixed Huffman blocks (3 bytes) + stored block start (1 byte)
                result.push(0x02); // BFINAL=0, BTYPE=01 (fixed Huffman)
                result.push(0x08); // end-of-block bits + BFINAL=0 + BTYPE=01 (2nd fixed Huffman)
                result.push(0x00); // end-of-block bits + next stored block header bits
                result.push(0x00); // LEN_lo=0 (after stored block skip-to-byte)
            }
            _ => unreachable!(),
        }
        *bridge_bytes = remainder;
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

/// Add PNG filter bytes. All rows use None filter (0x00).
fn add_filter_bytes(data: &[u8], row_width: usize) -> Vec<u8> {
    let mut filtered = Vec::with_capacity(data.len() / row_width * (row_width + 1));

    for chunk in data.chunks(row_width) {
        filtered.push(0x00);
        filtered.extend_from_slice(chunk);
    }

    filtered
}

/// Write IDAT chunk with stored deflate blocks.
///
/// All data (pixel rows + CD+EOCD rows) is in `filtered_data` as a single
/// contiguous buffer. It is split into stored DEFLATE blocks of up to 65535
/// bytes each, with the last block marked BFINAL=1.
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
    let num_chunks = filtered_data.chunks(IDAT_BLOCK_SIZE).count();
    for (i, chunk) in filtered_data.chunks(IDAT_BLOCK_SIZE).enumerate() {
        let is_last = i == num_chunks - 1;
        idat_content.push(if is_last { 0x01 } else { 0x00 });
        idat_content += &(chunk.len() as u16).to_le_bytes();
        idat_content += &(chunk.len() as u16).not().to_le_bytes();
        idat_content += chunk;
    }

    // Adler-32 over all decompressed data
    let adler = adler32(filtered_data);
    idat_content += &adler.to_be_bytes();

    let _ = write_png_chunk(buffer, b"IDAT", &idat_content);
}

/// Write a single central directory entry with a given extra field length.
fn write_cd_entry(buffer: &mut OutputBuffer, entry: &FileEntry, extra_len: u16) {
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
    *buffer += &extra_len.to_le_bytes();  // extra len
    *buffer += &0_u16.to_le_bytes();  // comment len
    *buffer += &0_u16.to_le_bytes();  // disk number
    *buffer += &0_u16.to_le_bytes();  // internal attrs
    *buffer += &0_u32.to_le_bytes();  // external attrs
    *buffer += &entry.header_offset.to_le_bytes();
    *buffer += entry.name.as_slice();
    // Extra field: all zeros (valid filter bytes and benign ZIP extra data)
    if extra_len > 0 {
        for _ in 0..extra_len {
            buffer.push(0);
        }
    }
}

/// Write central directory entries (no extra field padding).
fn write_central_directory(buffer: &mut OutputBuffer, entries: &[FileEntry]) {
    for entry in entries {
        write_cd_entry(buffer, entry, 0);
    }
}

/// Build CD+EOCD as contiguous bytes aligned so that every row boundary
/// (at `filtered_row_size` intervals from position 0 in the decompressed stream)
/// lands on a `0x00` byte — making it a valid PNG None filter.
///
/// ## CD entry layout (46 + name_len + extra_len bytes)
///
/// Bytes 0-3:   PK\x01\x02 (signature)
/// Bytes 4-33:  various fields (non-zero)
/// Bytes 34-41: disk_number(2) + internal_attrs(2) + external_attrs(4) = 8 × 0x00
/// Bytes 42-45: header_offset (4 bytes, usually non-zero)
/// Bytes 46+:   filename (name_len bytes, non-zero)
/// After name:  extra field (extra_len bytes, all zeros — fully controlled)
///
/// ## EOCD layout (22 bytes)
///
/// Bytes 0-3:   PK\x05\x06 (signature)
/// Bytes 4-7:   disk_number(2) + disk_with_cd(2) = 4 × 0x00
/// Bytes 8-21:  various fields (non-zero)
///
/// ## Strategy
///
/// For each CD entry, we choose an extra_len that ensures all row boundaries
/// falling within the entry (including its extra field) land on either:
/// - bytes 34-41 (the zero fields), or
/// - the extra field (all zeros)
///
/// The extra field bytes are fully controlled and set to 0x00, so any boundary
/// landing there is also a valid None filter byte.
///
/// For the EOCD, if a boundary would land in a non-zero region, we add padding
/// to the last CD entry's extra field to shift the EOCD's position.
fn build_aligned_cd_eocd(
    entries: &[FileEntry],
    filtered_row_size: usize,
    start_offset: usize,
    file_prefix_size: usize, // bytes in file before the decompressed stream starts
) -> Vec<u8> {
    let frs = filtered_row_size;

    // Single-pass algorithm: process entries sequentially, choosing each entry's
    // extra_len to position the NEXT entry (or EOCD) correctly.
    //
    // For a CD entry starting at position `pos`:
    //   - Row boundary falls at offset `frs - (pos % frs)` within the entry
    //     (if that offset < entry_total_size; otherwise no boundary)
    //   - Safe offsets: 34-41 (zero fields) or >= entry_fixed (extra field, all zeros)
    //
    // For EOCD starting at position `pos`:
    //   - Safe offsets: 4-7 (zero fields) or >= 22 (past EOCD, no boundary)
    //
    // Strategy: for each entry, compute valid start positions (pos % frs values)
    // where all boundaries land on safe bytes. Then set the previous entry's
    // extra_len (or pre_padding) to achieve a valid position.

    // Precompute valid CD start remainders for each entry
    // (depends on entry_fixed = 46 + name_len)
    let cd_valid_remainders: Vec<Vec<usize>> = entries.iter().map(|entry| {
        let entry_fixed = 46 + entry.name.len();
        valid_cd_remainders(entry_fixed, frs)
    }).collect();

    // Valid EOCD start remainders
    let eocd_valid = valid_eocd_remainders(frs);

    let mut extra_lens: Vec<usize> = vec![0; entries.len()];
    let mut pre_padding: usize = 0;

    // Position the first entry
    if !entries.is_empty() {
        let cur_remainder = (start_offset) % frs;
        if !cd_valid_remainders[0].contains(&cur_remainder) {
            // Find minimum padding to reach a valid remainder
            pre_padding = min_padding_to_valid(cur_remainder, &cd_valid_remainders[0], frs);
        }
    }

    let mut pos = start_offset + pre_padding;

    for i in 0..entries.len() {
        let entry_fixed = 46 + entries[i].name.len();

        // Determine what the NEXT structure needs
        let next_valid = if i + 1 < entries.len() {
            &cd_valid_remainders[i + 1]
        } else {
            &eocd_valid
        };

        // After this entry (with extra_len=0), the next structure starts at pos + entry_fixed
        let next_pos_base = pos + entry_fixed;
        let next_remainder = next_pos_base % frs;

        if !next_valid.contains(&next_remainder) {
            // Add extra_len to reach a valid remainder for the next structure
            extra_lens[i] = min_padding_to_valid(next_remainder, next_valid, frs);
        }

        pos = next_pos_base + extra_lens[i];
    }

    // Build the result
    let mut result: Vec<u8> = vec![0u8; pre_padding];

    // Compute the CD's file offset
    let cd_decompressed_offset = start_offset + pre_padding;
    let blocks_before_cd = cd_decompressed_offset.div_ceil(IDAT_BLOCK_SIZE);
    let cd_file_offset = file_prefix_size + cd_decompressed_offset + blocks_before_cd * 5;

    let mut buf = output_buffer();
    for (i, entry) in entries.iter().enumerate() {
        write_cd_entry(&mut buf, entry, extra_lens[i] as u16);
    }
    let cd_size = buf.len();
    write_eocd(&mut buf, entries.len() as u16, cd_size as u32, cd_file_offset as u32);
    result.extend_from_slice(&buf.into_bytes());

    // Pad to exact multiple of frs (relative to stream start)
    let total = start_offset + result.len();
    let remainder = total % frs;
    if remainder != 0 {
        let pad = frs - remainder;
        result.resize(result.len() + pad, 0);
    }

    result
}

/// Compute valid `pos % frs` values for a CD entry with the given fixed size.
///
/// A CD entry has zero bytes at offsets 34-41. The extra field (at offset
/// `entry_fixed` and beyond) is also all zeros. A row boundary at offset `d`
/// within the entry requires `d` to be in [34, 41] or >= entry_fixed.
///
/// For an entry at position `pos`, the first boundary offset is `frs - (pos % frs)`
/// (if that's < entry_fixed + any extra). Additional boundaries are at
/// `frs - (pos % frs) + k*frs`.
///
/// A remainder `r = pos % frs` is valid if ALL boundaries within the entry
/// land on safe bytes. Since extra_len can grow, we only need to check
/// boundaries that fall in [0, entry_fixed).
fn valid_cd_remainders(entry_fixed: usize, frs: usize) -> Vec<usize> {
    (0..frs).filter(|&r| {
        // Check all boundary offsets within [0, entry_fixed)
        // First boundary offset: (frs - r) % frs
        let first_off = if r == 0 { 0 } else { frs - r };
        if first_off >= entry_fixed {
            return true; // No boundary in fixed part
        }
        // Check this and subsequent boundaries
        let mut off = first_off;
        while off < entry_fixed {
            if !(34..=41).contains(&off) {
                return false;
            }
            off += frs;
        }
        true
    }).collect()
}

/// Compute valid `pos % frs` values for the EOCD (22 bytes, zeros at 4-7).
fn valid_eocd_remainders(frs: usize) -> Vec<usize> {
    (0..frs).filter(|&r| {
        let first_off = if r == 0 { 0 } else { frs - r };
        if first_off >= 22 {
            return true; // No boundary in EOCD
        }
        let mut off = first_off;
        while off < 22 {
            if !(4..=7).contains(&off) {
                return false;
            }
            off += frs;
        }
        true
    }).collect()
}

/// Find minimum padding to shift from `current_remainder` to any value in `valid`.
fn min_padding_to_valid(current_remainder: usize, valid: &[usize], frs: usize) -> usize {
    valid.iter()
        .map(|&v| (v + frs - current_remainder) % frs)
        .filter(|&p| p > 0) // Must add at least something (current is invalid)
        .min()
        .unwrap_or(frs) // Fallback: shift by full frs
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
        // For indexed color (1 byte per pixel), alignment is just DATA_ALIGNMENT (64)
        assert_eq!(calculate_row_width(100, BASE_MIN_ROW_WIDTH, 1), BASE_MIN_ROW_WIDTH);
        assert_eq!(BASE_MIN_ROW_WIDTH, 68);

        // 10KB: sqrt(10000) = 100, data portion aligned to 128 → row_width = 132
        let w = calculate_row_width(10_000, BASE_MIN_ROW_WIDTH, 1);
        assert_eq!(w, 132, "Expected 132 (128 data + 4 header), got {}", w);

        // 40KB: sqrt(40000) ≈ 200, data portion aligned to 256 → row_width = 260
        let w = calculate_row_width(40_000, BASE_MIN_ROW_WIDTH, 1);
        assert_eq!(w, 260, "Expected 260 (256 data + 4 header), got {}", w);

        // With smaller min_width requirement → still gets 68 minimum
        let w = calculate_row_width(100, 60, 1);
        assert_eq!(w, 68, "Expected min_width of 68 (64 data aligned), got {}", w);

        // Verify data portion alignment (row_width - 4 should be multiple of 64)
        assert_eq!((w - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "Data portion should be aligned to {}", DATA_ALIGNMENT);

        // For RGB (3 bytes per pixel): row_width % 3 == 0 AND (row_width - 4) % 64 == 0
        let w = calculate_row_width(10_000, BASE_MIN_ROW_WIDTH, 3);
        assert_eq!((w - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "RGB data portion should be aligned to 64");
        assert_eq!(w % 3, 0, "RGB row_width must be divisible by bytes_per_pixel");

        // For RGBA (4 bytes per pixel): row_width % 4 == 0 AND (row_width - 4) % 64 == 0
        let w = calculate_row_width(10_000, BASE_MIN_ROW_WIDTH, 4);
        assert_eq!((w - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "RGBA data portion should be aligned to 64");
        assert_eq!(w % 4, 0, "RGBA row_width must be divisible by bytes_per_pixel");
    }

    #[test]
    fn test_min_row_width_for_files() {
        // Short filenames - returns BASE_MIN_ROW_WIDTH (68)
        let files = vec![(b"a.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files, 1);
        assert_eq!(min, BASE_MIN_ROW_WIDTH); // 30 + 5 + 4 = 39 < 68, so 68

        // Moderately long filename - still fits in 68
        let files = vec![(b"this-is-a-very-long-filename.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files, 1);
        // 30 + 32 + 4 = 66 < 68, so still 68
        assert_eq!(min, 68, "Expected 68 (66 fits in minimum), got {}", min);

        // Very long filename that requires wider rows
        let files = vec![(b"this-is-an-extremely-long-filename-that-exceeds-minimum.txt".as_ref(), b"data".as_ref())];
        let min = min_row_width_for_files(&files, 1);
        // 30 + 58 + 4 = 92 → data aligned to 128 → row_width = 132
        assert_eq!(min, 132, "Expected 132 (128 data + 4 header), got {}", min);

        // Verify data portion alignment
        assert_eq!((min - DEFLATE_HEADER_OVERHEAD) % DATA_ALIGNMENT, 0,
            "Data portion should be aligned to {}", DATA_ALIGNMENT);
    }

    #[test]
    fn test_polyglot_structure() {
        let files = vec![(b"test.txt".as_ref(), b"Hello, World!".as_ref())];
        let result = build_polyglot(&files, 0, BitDepth::EightBit, LIGHTNESS, None);

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

        let result = build_polyglot(&files, 0, BitDepth::EightBit, LIGHTNESS, None);

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

        let result = build_polyglot(&file_refs, 0, BitDepth::EightBit, LIGHTNESS, None);

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

        let label = render_filename_label(name, row_width, &font, &header, false, 0, body.len(), 1);

        // Bottom row (padding below text) should have header bytes - stretch starts there
        // label_rows = max_height + 2
        let label_rows = font.max_height() + 2;
        let last_row_start = (label_rows - 1) * row_width;
        assert_eq!(&label[last_row_start..last_row_start + 4], b"PK\x03\x04",
            "Label should have PK signature in bottom row (stretch starts there)");
    }
