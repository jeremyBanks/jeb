use bitvec::prelude::Lsb0;
use bitvec::prelude::Msb0;
use bitvec::vec::BitVec;
use bitvec::view::AsBits;
use indexmap::IndexMap;
use zipng::font::Font;
use zipng::font::Mini5pt;
use zipng::font::FONTS;
use zipng::generic::panic;
use zipng::png::write_png;
use zipng::png::BitDepth::EightBit;
use zipng::png::BitDepth::OneBit;
use zipng::png::ColorMode::Indexed;
use zipng::png::ColorMode::RedGreenBlue;
use zipng::png::PALLETTE_8_BIT_DATA;
use zipng::zip;
use zipng::PngOptions;
use zipng::ZipngOptions;

fn main() -> Result<(), panic> {
    let mut data = Vec::new();

    let font = Mini5pt;
    let width = font.width();
    let bits = font.width() * font.height();
    for (character, bitmap) in font.glyphs() {
        // if *character != 'A' {
        //     continue;
        // }
        let mut v: Vec<_> = bitmap
            .to_le_bytes()
            .as_bits::<Lsb0>()
            .iter()
            .take(bits)
            .map(|bit| if *bit { 0xFFu8 } else { 0x00 })
            .collect();
        v.reverse();

        data.extend(v);
        data.extend(vec![0; width]);
    }

    let mut buffer = Vec::new();

    let ZipngOptions {
        png:
            PngOptions {
                bit_depth,
                color_mode,
                color_palette,
                // width,
                ..
            },
        ..
    } = ZipngOptions::default_for_data(&data);
    let color_palette = color_palette.as_deref();

    let bit_depth = EightBit;
    let color_mode = Indexed;
    let color_palette = Some(PALLETTE_8_BIT_DATA.as_slice());

    // let bit_depth = EightBit;
    // let color_mode = RedGreenBlue;
    // let color_palette = None::<&[u8]>;

    // let bit_depth = OneBit;
    // let color_mode = Indexed;
    // let color_palette = [0xFF_u8, 0xFF, 0xEE, 0x11, 0x11, 0x33];
    // let color_palette = Some(&color_palette[..]);

    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let pixels = data.len() * 8 / bits_per_pixel;

    let height = (pixels / width).max(1);

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
