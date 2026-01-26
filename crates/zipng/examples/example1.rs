use indexmap::IndexMap;
use zipng::generic::panic;
use zipng::png::write_png;
use zipng::png::BitDepth::EightBit;
use zipng::png::ColorMode::Indexed;
use zipng::png::PALLETTE_8_BIT_DATA;
use zipng::zip;

fn main() -> Result<(), panic> {
    let files: [(&[u8], &[u8]); 4] = [
        (b"mimetype".as_ref(), b"zip/zip".as_ref()),
        (b"README.md".as_ref(), b"welcome".as_ref()),
        (
            b"assets/icon.png".as_ref(),
            include_bytes!("../../../icon.png"),
        ),
        (
            b"assets/png.exe".as_ref(),
            include_bytes!("../../../target/debug/examples/example1.exe"),
        ),
    ];
    let files = IndexMap::from_iter(files.iter().map(|(k, v)| (k.to_vec(), v.to_vec())));

    let data = zip(&files.into());

    let mut buffer = Vec::new();
    let width = 1024;
    write_png(
        &mut buffer,
        &data,
        width,
        (data.len() / width as usize).try_into()?,
        EightBit,
        Indexed,
        Some(PALLETTE_8_BIT_DATA),
    );

    std::fs::write("../../target/test.png", buffer)?;

    Ok(())
}
