use std::collections::HashMap;

fn main() {
    // Load Sky font manually
    let json_data = include_str!("../src/text/sky.json");
    let png_data = include_bytes!("../src/text/sky.png");
    
    let meta: FontMeta = serde_json::from_str(json_data).unwrap();
    println!("Font: {} ({}x{})", meta.name, meta.w, meta.h);
    
    let img = image::load_from_memory(png_data).unwrap().to_luma8();
    
    // Extract 'A' glyph (row 4, col 1 based on rows array)
    // rows[4] = "@ABCDEFG"
    let row_idx = 4;
    let col_idx = 1; // 'A' is at index 1
    
    let glyph_x = meta.x + col_idx * meta.dx;
    let glyph_y = meta.y + row_idx * meta.dy;
    
    println!("\n'A' glyph at ({}, {}):", glyph_x, glyph_y);
    for py in 0..meta.h {
        let mut row_str = String::new();
        for px in 0..meta.w {
            let pixel = img.get_pixel((glyph_x + px) as u32, (glyph_y + py) as u32).0[0];
            row_str.push(if pixel < 128 { '#' } else { '.' });
        }
        println!("  {}", row_str);
    }
    
    // Check 'V' glyph (row 5, col 6)
    // rows[5] = "HIJKLMNO" - no, V is in row 7
    // rows[7] = "XYZ[\\]^_" - no
    // Let me find V: rows[5] = "HIJKLMNO", rows[6] = "PQRSTUVW"
    let row_idx = 6;
    let col_idx = 5; // V is at index 5 in "PQRSTUVW"
    
    let glyph_x = meta.x + col_idx * meta.dx;
    let glyph_y = meta.y + row_idx * meta.dy;
    
    println!("\n'V' glyph at ({}, {}):", glyph_x, glyph_y);
    for py in 0..meta.h {
        let mut row_str = String::new();
        for px in 0..meta.w {
            let pixel = img.get_pixel((glyph_x + px) as u32, (glyph_y + py) as u32).0[0];
            row_str.push(if pixel < 128 { '#' } else { '.' });
        }
        println!("  {}", row_str);
    }
}

#[derive(serde::Deserialize)]
struct FontMeta {
    x: usize, y: usize, dx: usize, dy: usize, w: usize, h: usize,
    rows: Vec<String>, name: String,
}
