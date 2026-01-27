use indexmap::IndexMap;
use zipng::generic::panic;
use zipng::png::write_png;
use zipng::png::BitDepth::EightBit;
use zipng::png::ColorMode::Indexed;
use zipng::png::ColorMode::RedGreenBlue;
use zipng::png::PALLETTE_8_BIT_DATA;
use zipng::zip;
use zipng::PngOptions;
use zipng::ZipngOptions;

fn main() -> Result<(), panic> {
    let files: [(&[u8], &[u8]); 3] = [
        (
            b"assets/Cargo.toml".as_ref(),
            include_bytes!("../Cargo.toml"),
        ),
        (
            b"assets/Cargo.lock".as_ref(),
            include_bytes!("../Cargo.lock"),
        ),
        (b"assets/a.png".as_ref(), include_bytes!("../icon.png")),
    ];
    let files = IndexMap::from_iter(files.iter().map(|(k, v)| (k.to_vec(), v.to_vec())));

    let data = zip(&files.into());

    let mut buffer = Vec::new();

    let ZipngOptions {
        png:
            PngOptions {
                bit_depth,
                color_mode,
                color_palette,
                width,
                ..
            },
        ..
    } = ZipngOptions::default_for_data(&data);
    let color_palette = color_palette.as_deref();

    // let bit_depth = EightBit;
    // let color_mode = Indexed;
    // let color_palette = Some(PALLETTE_8_BIT_DATA.as_slice());

    // let bit_depth = EightBit;
    // let color_mode = RedGreenBlue;
    // let color_palette = None::<&[u8]>;

    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let pixels = data.len() * 8 / bits_per_pixel;

    let height = pixels / width;

    write_png(
        &mut buffer,
        &data,
        width as u32,
        height as u32,
        bit_depth,
        color_mode,
        color_palette,
    );

    std::fs::write("target/test.png", buffer)?;

    Ok(())
}
