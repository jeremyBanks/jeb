// Test kerning calculation
fn main() {
    // Simulate two simple glyphs
    // Glyph 1: pixels on right edge
    let glyph1 = vec![
        vec![false, false, true],
        vec![false, false, true],
        vec![false, false, true],
    ];
    // Glyph 2: pixels on left edge  
    let glyph2 = vec![
        vec![true, false, false],
        vec![true, false, false],
        vec![true, false, false],
    ];
    
    let width = 3;
    
    // Test glyphs_touch at different offsets
    for gap in 0..6 {
        let offset = (width + gap) as i32;
        let touches = glyphs_touch(&glyph1, &glyph2, offset);
        println!("gap={}, offset={}, touches={}", gap, offset, touches);
    }
    
    let min_gap = min_glyph_gap(&glyph1, &glyph2, width);
    println!("\nmin_glyph_gap = {}", min_gap);
}

fn glyphs_touch(prev: &[Vec<bool>], next: &[Vec<bool>], offset: i32) -> bool {
    for (prev_row, prev_pixels) in prev.iter().enumerate() {
        for (prev_col, &prev_on) in prev_pixels.iter().enumerate() {
            if !prev_on { continue; }
            for (next_row, next_pixels) in next.iter().enumerate() {
                for (next_col, &next_on) in next_pixels.iter().enumerate() {
                    if !next_on { continue; }
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

fn min_glyph_gap(prev: &[Vec<bool>], next: &[Vec<bool>], glyph_width: usize) -> usize {
    for gap in 0..(glyph_width * 2) {
        let offset = (glyph_width + gap) as i32;
        if !glyphs_touch(prev, next, offset) {
            return gap;
        }
    }
    glyph_width
}
