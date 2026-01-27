//! Grayscale 8-bit PNG example - creates a grayscale gradient image.

use std::fs;
use zipng::{
    palettes::mappings::BIT_COUNT,
    panic,
    EightBit,
    Png,
};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    let mut png = Png::new_grayscale(512, 128, EightBit);

    for y in 0..png.height {
        for x in 0..png.width {
            png.set_pixel(
                x,
                y,
                &[BIT_COUNT[(x * 4 / 9 + x.abs_diff(y) / 15).min(255)]],
            )?;
        }
    }

    let output = png.serialize();
    fs::write("target/greyscale8bit.png", AsRef::<[u8]>::as_ref(&output))?;
    println!("Created target/greyscale8bit.png");

    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
