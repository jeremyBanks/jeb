//! RGB PNG example - creates a colorful gradient image.

use std::fs;
use zipng::{panic, Png};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    let mut png = Png::new_rgb(512, 128);

    for y in 0..png.height {
        for x in 0..png.width {
            png.set_pixel(x, y, &[
                (x / 4) as u8,
                y as u8,
                (x * 4 / 9 + x.abs_diff(y) / 15).min(255) as u8,
            ])?;
        }
    }

    // Draw diagonal color stripes
    for x in 0..png.height {
        png.set_pixel(png.height * 2 - x, x, &[0x00, 0, 0])?;
        png.set_pixel(png.height * 2 - x + 1, x, &[0xFF, 0, 0])?;
        png.set_pixel(png.height * 2 - x + 2, x, &[0xFF, 0xFF, 0])?;
        png.set_pixel(png.height * 2 - x + 3, x, &[0, 0xFF, 0])?;
        png.set_pixel(png.height * 2 - x + 4, x, &[0, 0xFF, 0xFF])?;
        png.set_pixel(png.height * 2 - x + 5, x, &[0, 0, 0xFF])?;
        png.set_pixel(png.height * 2 - x + 6, x, &[0xFF, 0, 0xFF])?;
        png.set_pixel(png.height * 2 - x + 7, x, &[0xFF, 0xFF, 0xFF])?;
    }

    let output = png.serialize();
    fs::write("target/rgb.png", AsRef::<[u8]>::as_ref(&output))?;
    println!("Created target/rgb.png");

    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
