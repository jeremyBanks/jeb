//! Renders all bitmap fonts side-by-side in a grid PNG for visual comparison.

use std::collections::BTreeSet;

use zipng::polyglot::fonts::{ALL_FONTS, BitmapFont};

fn main() {
    // Collect all fonts.
    let mut fonts: Vec<&BitmapFont> = ALL_FONTS.iter().map(|f| &***f).collect();

    // Union of all characters across all fonts (excluding space).
    let mut all_chars = BTreeSet::new();
    for font in &fonts {
        for c in font.chars() {
            if c != ' ' {
                all_chars.insert(c);
            }
        }
    }
    let chars: Vec<char> = all_chars.into_iter().collect();

    // Sort by area (width * height), then by name.
    fonts.sort_by_key(|f| (f.width * f.height, f.name));

    // Cell dimensions based on largest font.
    let max_w = fonts.iter().map(|f| f.width).max().unwrap();
    let max_h = fonts.iter().map(|f| f.height).max().unwrap();
    let cell_w = max_w + 1;
    let cell_h = max_h + 1;

    // 4px white padding around the outside.
    let pad = 4;

    // Estimate name area width generously.
    let name_area_w = 200;
    let img_w = pad + chars.len() * cell_w + name_area_w + pad;
    let img_h = pad + fonts.len() * cell_h + pad;

    let mut pixels = vec![255u8; img_w * img_h];

    for (row, font) in fonts.iter().enumerate() {
        let y0 = pad + row * cell_h;

        // Render each character cell.
        for (col, &c) in chars.iter().enumerate() {
            let x0 = pad + col * cell_w;

            if let Some(lookup) = font.get_glyph(c) {
                let is_native = font.chars().any(|fc| fc == c);
                if is_native {
                    for (gy, grow) in lookup.glyph.iter().enumerate() {
                        for (gx, &on) in grow.iter().enumerate() {
                            if on {
                                let px = x0 + gx;
                                let py = y0 + gy;
                                if px < img_w && py < img_h {
                                    pixels[py * img_w + px] = 0;
                                }
                            }
                        }
                    }
                } else {
                    draw_checkerboard(&mut pixels, img_w, x0, y0, max_w, max_h);
                }
            } else {
                draw_checkerboard(&mut pixels, img_w, x0, y0, max_w, max_h);
            }
        }

        // Render font name using layout_text.
        let name_x0 = pad + chars.len() * cell_w + 2;
        let (canvas, _positions, _width) = font.layout_text(font.name);
        for (gy, canvas_row) in canvas.iter().enumerate() {
            for (gx, &on) in canvas_row.iter().enumerate() {
                if on {
                    let px = name_x0 + gx;
                    let py = y0 + gy;
                    if px < img_w && py < img_h {
                        pixels[py * img_w + px] = 0;
                    }
                }
            }
        }
    }

    // Save as PNG.
    let img = image::GrayImage::from_raw(img_w as u32, img_h as u32, pixels)
        .expect("image buffer size mismatch");
    let path = "target/font_grid.png";
    img.save(path).expect("failed to save PNG");
    println!("Saved {path} ({img_w}×{img_h})");
}

fn draw_checkerboard(pixels: &mut [u8], stride: usize, x0: usize, y0: usize, w: usize, h: usize) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x0 + dx;
            let py = y0 + dy;
            if (dx + dy) % 2 == 0 {
                pixels[py * stride + px] = 0;
            }
        }
    }
}
