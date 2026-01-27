//! Text rendering example - renders font glyphs to a PNG.

use std::fs;
use bitvec::prelude::Lsb0;
use bitvec::view::AsBits;
use zipng::font::{Font, Mini5pt};
use zipng::palettes::viridis::VIRIDIS;
use zipng::{panic, EightBit, Png};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    let mut data = Vec::new();

    let font = Mini5pt;
    let glyph_width = font.width();
    let bits = font.width() * font.height();

    for (_character, bitmap) in font.glyphs() {
        let mut v: Vec<_> = bitmap
            .to_le_bytes()
            .as_bits::<Lsb0>()
            .iter()
            .take(bits)
            .map(|bit| if *bit { 0xFFu8 } else { 0x00 })
            .collect();
        v.reverse();

        data.extend(v);
        data.extend(vec![0; glyph_width]);
    }

    // Calculate dimensions
    let width = glyph_width * 2;  // Two glyphs wide
    let pixels = data.len();
    let height = (pixels / width).max(1);

    // Pad data to fill complete image
    data.resize(width * height, 0);

    // Create indexed PNG with VIRIDIS palette
    let mut png = Png::new_indexed(width, height, EightBit, VIRIDIS);

    // Copy pixel data
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if idx < data.len() {
                png.set_pixel(x, y, &[data[idx]])?;
            }
        }
    }

    let output = png.serialize();
    fs::write("target/text.png", AsRef::<[u8]>::as_ref(&output))?;
    println!("Created target/text.png ({} glyphs, {}x{} pixels)",
             font.glyphs().len(), width, height);

    Ok(())
}
