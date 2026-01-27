use {
    crate::{
        crc32, output_buffer, panic, write_zlib, BitDepth, ColorType, OutputBuffer, PNG_HEADER_SIZE,
    },
    std::ops::AddAssign,
    tracing::warn,
};

pub fn write_png_header(
    buffer: &mut OutputBuffer,
    width: u32,
    height: u32,
    color_depth: BitDepth,
    color_mode: ColorType,
) -> Result<usize, panic> {
    let before = buffer.offset();
    if before != 0 {
        warn!("PNG header is being written at nonzero stream offset: {before}");
    }

    // 0x0000..0x0008: PNG signature
    buffer
        .tagged("png", "signature")
        .add_assign(b"\x89PNG\r\n\x1A\n");

    // 0x0008..0x0010:     IHDR chunk prefix (length and type)
    write_png_chunk(&mut buffer.tagged("png", "header"), b"IHDR", &{
        let mut data = output_buffer();

        // 0x0010..0x0014: pixel width
        data.tagged("png", "width").add_assign(&width.to_be_bytes());
        // 0x0004..0x0018: pixel height
        data.tagged("png", "height")
            .add_assign(&height.to_be_bytes());
        // 0x0018:         color bit depth
        data.tagged("png", "bit-depth")
            .add_assign(&u8::from(color_depth).to_be_bytes());
        // 0x0019:         color type: grayscale
        data.tagged("png", "color-mode")
            .add_assign(&u8::from(color_mode).to_be_bytes());
        // 0x001A:         compression method: deflate
        data.tagged("png", "compression-mode")
            .add_assign(&0_u8.to_be_bytes());
        // 0x001B:         filter method: basic
        data.tagged("png", "filter-mode")
            .add_assign(&0_u8.to_be_bytes());
        // 0x001C:         interlace method: none
        data.tagged("png", "interlace-mode")
            .add_assign(&0_u8.to_be_bytes());

        data
        // 0x001D..0x0025: IHDR chunk suffix (checksum)
    })?;

    let after = buffer.offset();

    assert_eq!(after - before, PNG_HEADER_SIZE);

    Ok(PNG_HEADER_SIZE)
}

pub fn write_png_palette(buffer: &mut OutputBuffer, palette: &[u8]) -> Result<usize, panic> {
    write_png_chunk(
        &mut buffer.tagged("png", "palette"),
        b"PLTE",
        &OutputBuffer::with_tag(palette, "png", "colors"),
    )
}

pub fn write_png_body(buffer: &mut OutputBuffer, data: &[u8]) -> Result<usize, panic> {
    let mut deflated = output_buffer();
    write_zlib(&mut deflated.tagged("png", "zlib"), data)?;

    write_png_chunk(&mut buffer.tagged("png", "pixels"), b"IDAT", &deflated)
}

pub fn write_non_png_chunk(buffer: &mut OutputBuffer, data: &[u8]) -> Result<usize, panic> {
    write_png_chunk(
        &mut buffer.tagged("png", "comment"),
        b"pkPK",
        &OutputBuffer::without_tag(data),
    )
}

pub fn write_png_footer(buffer: &mut OutputBuffer) -> Result<usize, panic> {
    write_png_chunk(
        &mut buffer.tagged("png", "footer"),
        b"IEND",
        &OutputBuffer::new(),
    )
}

pub fn write_png_chunk(
    buffer: &mut OutputBuffer,
    chunk_type: &[u8; 4],
    data: &OutputBuffer,
) -> Result<usize, panic> {
    let before = buffer.offset();

    let mut buffer = buffer.tagged("png", "chunk");

    buffer.tagged("png", "size").add_assign(
        &u32::try_from(data.len())
            .expect("png chunk larger than 2GiB")
            .to_be_bytes(),
    );
    buffer.tagged("png", "type").add_assign(chunk_type);
    buffer.tagged("png", "body").add_assign(data);
    buffer.tagged("png", "checksum").add_assign(
        &crc32(
            &[chunk_type.as_slice(), data.as_ref()]
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
        )
        .to_be_bytes(),
    );

    let after = buffer.offset();
    Ok(after - before)
}

pub fn write_png(
    buffer: &mut OutputBuffer,
    pixel_data: &[u8],
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    color_mode: ColorType,
    palette: Option<&[u8]>,
) -> Result<(), panic> {
    let buffer = &mut *buffer.tagged("png", "png");

    write_png_header(buffer, width, height, bit_depth, color_mode)?;
    if let Some(palette) = palette {
        write_png_palette(buffer, palette)?;
    }
    // We need to insert a 0x00 byte at the start of every line (every `width`
    // bytes) to indicate that the line is not filtered.
    let mut filtered_data = output_buffer();

    let bits_per_pixel = bit_depth.bits_per_sample() * color_mode.samples_per_pixel();
    let bits_per_line = width * bits_per_pixel as u32;
    let bytes_per_line = (bits_per_line + 7) / 8;

    for (i, byte) in pixel_data.iter().enumerate() {
        if i % (bytes_per_line as usize) == 0 {
            filtered_data.tagged("png", "filter").push(0x00);
        }
        filtered_data.tagged("png", "row").push(*byte);
    }
    write_png_body(buffer, filtered_data.as_ref())?;
    write_png_footer(buffer)?;
    Ok(())
}
