use std::collections::HashMap;

fn main() {
    let fonts: Vec<(&str, &[u8], &[u8])> = vec![
        ("sixth", include_bytes!("../src/text/sixth.json"), include_bytes!("../src/text/sixth.png")),
        ("swiss", include_bytes!("../src/text/swiss.json"), include_bytes!("../src/text/swiss.png")),
        ("sky", include_bytes!("../src/text/sky.json"), include_bytes!("../src/text/sky.png")),
        ("monte", include_bytes!("../src/text/monte.json"), include_bytes!("../src/text/monte.png")),
        ("sugimori", include_bytes!("../src/text/sugimori.json"), include_bytes!("../src/text/sugimori.png")),
        ("mini", include_bytes!("../src/text/mini.json"), include_bytes!("../src/text/mini.png")),
        ("micro", include_bytes!("../src/text/micro.json"), include_bytes!("../src/text/micro.png")),
    ];

    for (name, json_data, png_data) in fonts {
        audit_font(name, json_data, png_data);
    }
}

#[derive(serde::Deserialize)]
struct FontMeta {
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
    w: usize,
    h: usize,
    #[serde(default)]
    b: usize,
    rows: Vec<String>,
    name: String,
}

fn audit_font(file_name: &str, json_data: &[u8], png_data: &[u8]) {
    let meta: FontMeta = serde_json::from_slice(json_data).unwrap();
    let img = image::load_from_memory(png_data).unwrap().to_luma8();
    let (img_w, img_h) = (img.width() as usize, img.height() as usize);

    println!("=== {} ({}) ===", meta.name, file_name);
    println!("    image: {}x{}, grid: x={} y={} dx={} dy={} w={} h={} b={}",
        img_w, img_h, meta.x, meta.y, meta.dx, meta.dy, meta.w, meta.h, meta.b);

    // Build map of which pixels belong to which glyph cell
    let mut glyph_cells: HashMap<(usize, usize), (usize, usize, char)> = HashMap::new();
    for (row_idx, row_str) in meta.rows.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            let gx = meta.x + col_idx * meta.dx;
            let gy = meta.y + row_idx * meta.dy;
            for py in 0..meta.h {
                for px in 0..meta.w {
                    glyph_cells.insert((gx + px, gy + py), (row_idx, col_idx, ch));
                }
            }
        }
    }

    // Find all dark pixels
    let mut dark_outside: Vec<(usize, usize)> = Vec::new();
    let mut glyph_dark_pixels: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for y in 0..img_h {
        for x in 0..img_w {
            let pixel = img.get_pixel(x as u32, y as u32).0[0];
            if pixel < 128 {
                if let Some(&(row, col, _)) = glyph_cells.get(&(x, y)) {
                    glyph_dark_pixels.entry((row, col)).or_default().push((x, y));
                } else {
                    dark_outside.push((x, y));
                }
            }
        }
    }

    if !dark_outside.is_empty() {
        println!("  *** {} dark pixels OUTSIDE any glyph cell ***", dark_outside.len());
        // Group by approximate glyph location to identify which glyph is bleeding
        for &(x, y) in dark_outside.iter().take(50) {
            // Find nearest glyph
            let approx_col = if x >= meta.x { (x - meta.x) / meta.dx } else { 0 };
            let approx_row = if y >= meta.y { (y - meta.y) / meta.dy } else { 0 };
            let ch = meta.rows.get(approx_row)
                .and_then(|r| r.chars().nth(approx_col))
                .unwrap_or('?');
            let gx = meta.x + approx_col * meta.dx;
            let gy = meta.y + approx_row * meta.dy;
            println!("    pixel ({},{}) outside cell, nearest glyph '{}' at ({},{}) cell=[{},{}..{},{}]",
                x, y, ch, gx, gy, gx, gy, gx + meta.w - 1, gy + meta.h - 1);
        }
        if dark_outside.len() > 50 {
            println!("    ... and {} more", dark_outside.len() - 50);
        }
    }

    // Check for overlapping glyph cells
    let mut overlaps = Vec::new();
    for (row_i, row_str_i) in meta.rows.iter().enumerate() {
        for (col_i, ch_i) in row_str_i.chars().enumerate() {
            let gx_i = meta.x + col_i * meta.dx;
            let gy_i = meta.y + row_i * meta.dy;
            for (row_j, row_str_j) in meta.rows.iter().enumerate() {
                for (col_j, ch_j) in row_str_j.chars().enumerate() {
                    if (row_j, col_j) <= (row_i, col_i) { continue; }
                    let gx_j = meta.x + col_j * meta.dx;
                    let gy_j = meta.y + row_j * meta.dy;
                    // Check if cells overlap
                    let x_overlap = gx_i < gx_j + meta.w && gx_j < gx_i + meta.w;
                    let y_overlap = gy_i < gy_j + meta.h && gy_j < gy_i + meta.h;
                    if x_overlap && y_overlap {
                        overlaps.push((row_i, col_i, ch_i, row_j, col_j, ch_j));
                    }
                }
            }
        }
    }
    if !overlaps.is_empty() {
        println!("  *** {} overlapping glyph cell pairs ***", overlaps.len());
        for &(ri, ci, chi, rj, cj, chj) in overlaps.iter().take(20) {
            println!("    '{}' (row={},col={}) overlaps '{}' (row={},col={})", chi, ri, ci, chj, rj, cj);
        }
        if overlaps.len() > 20 {
            println!("    ... and {} more", overlaps.len() - 20);
        }
    }

    // Check for empty non-space glyphs
    let mut empty_glyphs = Vec::new();
    for (row_idx, row_str) in meta.rows.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            if ch == ' ' { continue; }
            if !glyph_dark_pixels.contains_key(&(row_idx, col_idx)) {
                empty_glyphs.push((row_idx, col_idx, ch));
            }
        }
    }
    if !empty_glyphs.is_empty() {
        println!("  *** {} non-space glyphs with NO dark pixels ***", empty_glyphs.len());
        for &(row, col, ch) in &empty_glyphs {
            println!("    '{}' (U+{:04X}) row={} col={}", ch, ch as u32, row, col);
        }
    }

    if dark_outside.is_empty() && overlaps.is_empty() && empty_glyphs.is_empty() {
        println!("  All glyphs OK.");
    }
    println!();
}
