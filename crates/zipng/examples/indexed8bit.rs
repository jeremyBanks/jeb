//! Indexed 8-bit PNG example - creates a colorful image using the TURBO palette.

use std::fs;
use zipng::{
    palettes::singles::TURBO,
    panic,
    BitDepth::EightBit,
    Png,
};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    let mut png = Png::new_indexed(512, 128, EightBit, TURBO);

    for y in 0..png.height {
        for x in 0..png.width {
            png.set_pixel(x, y, &[(x * 4 / 9 + x.abs_diff(y) / 15).min(255) as u8])?;
        }
    }

    let output = png.serialize();
    fs::write("target/indexed8bit.png", AsRef::<[u8]>::as_ref(&output))?;
    println!("Created target/indexed8bit.png");

    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
