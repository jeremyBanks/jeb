use {
    crate::{
        crc32,
        io::{write_aligned_pad_end, write_aligned_pad_start},
        output_buffer, panic, OutputBuffer,
    },
    bstr::ByteSlice,
    std::io::Write,
};

const BLOCK_SIZE: usize = 1024;

pub fn write_zip(
    output: &mut OutputBuffer,
    files: &[(&[u8], &[u8])],
    suffix: &[u8],
) -> Result<usize, panic> {
    if suffix.find(b"PK\x05\x06").is_some() {
        panic!("Zip file suffix must not contain zip file terminator signature PK\\x05\\x06");
    }

    let output = &mut *output.tagged("zip", "zip");

    let start = output.offset();
    let mut files_with_offsets = Vec::with_capacity(files.len());

    for (name, body) in files.iter() {
        let mut header = output_buffer();
        let mut header = &mut *header.tagged("zip", "head");
        // 0x0000..0x0004: local file header signature
        *header.tagged("png", "signature") += b"PK\x03\x04";
        // 0x0004..0x0006: version needed to extract
        header += &1_0_u16.to_le_bytes();
        // 0x0006..0x0008: general purpose bit flag
        header += &[0x00; 2];
        // 0x0008..0x000A: compression method
        header += &[0x00; 2];
        // 0x000A..0x000C: modification time
        header += b"PK";
        // 0x000C..0x000E: modification date
        header += b"PK";
        // 0x000E..0x0012: checksum
        header += &crc32(body).to_le_bytes();
        // 0x0012..0x0016: compressed size
        header += &u32::try_from(body.len())
            .expect("file size larger than 4GiB")
            .to_le_bytes();
        // 0x0016..0x001A: uncompressed size
        header += &u32::try_from(body.len())
            .expect("file size larger than 4GiB")
            .to_le_bytes();
        // 0x001A..0x001E: file name length
        header += &u16::try_from(name.len())
            .expect("file name larger than 64KiB")
            .to_le_bytes();
        // 0x001E..0x0022: extra fields length
        header += &[0x00; 2];
        // 0x0022: file name, followed by extra fields (we have none)
        header += *name;
        let before = output.offset();
        if !body.is_empty() && name != b"mimetype" {
            write_aligned_pad_start(output, header.as_ref(), BLOCK_SIZE)?;
            write_aligned_pad_end(output, body, BLOCK_SIZE)?;
        } else {
            *output.tagged("zip", "file-header") += &*header;
            *output.tagged("zip", "file-body") += *body;
        };
        files_with_offsets.push((*name, *body, before));
    }

    let mut central_directory = output_buffer();
    let mut central_directory = &mut *central_directory.tagged("zip", "index");
    for (name, body, header_offset) in files_with_offsets.iter() {
        let name = name.to_vec();
        let name_length = u16::try_from(name.len()).expect("file name larger than 64KiB");
        let body_length = u32::try_from(body.len()).expect("file size larger than 4GiB");
        let header_offset = u32::try_from(*header_offset).expect("archive larger than 4GiB");
        let crc = crc32(body).to_le_bytes();
        let mut header = output_buffer();
        let mut header = &mut *header.tagged("zip", "head");
        // 0x0000..0x0004: central file header signature
        header += b"PK\x01\x02";
        // 0x0004..0x0006: creator version and platform
        header += &1_0_u16.to_le_bytes();
        // 0x0006..0x0008: required version
        header += &1_0_u16.to_le_bytes();
        // 0x0008..0x000A: general purpose bit flag
        header += &[0x00; 2];
        // 0x000A..0x000C: compression method
        header += &[0x00; 2];
        // 0x000C..0x000E: modification time
        header += b"PK";
        // 0x000E..0x0010: modification date
        header += b"PK";
        // 0x0010..0x0014: checksum
        header += &crc;
        // 0x0014..0x0018: compressed size
        header += &body_length.to_le_bytes();
        // 0x0018..0x001C: uncompressed size
        header += &body_length.to_le_bytes();
        // 0x001C..0x001E: file name length
        header += &name_length.to_le_bytes();
        // 0x001E..0x0020: extra field length
        header += &[0x00; 2];
        // 0x0020..0x0022: file comment length
        header += &[0x00; 2];
        // 0x0022..0x0024: disk number
        header += &[0x00; 2];
        // 0x0024..0x0026: internal file attributes
        header += &[0x00; 2];
        // 0x0026..0x002A: external file attributes
        header += &[0x00; 4];
        // 0x002A..0x002E: local file header offset from start of archive
        header += &header_offset.to_le_bytes();
        // 0x002E..: file name, followed by extra fields and comments (we have none)
        header += &*name;

        central_directory += &*header;
    }

    let directory_terminator_len = 22 + suffix.len();

    let before_central_directory = output.offset();
    *output.tagged("zip", "index") += &*central_directory;
    let central_directory_len_without_terminator = output.offset() - before_central_directory;

    let _after_central_directory = directory_terminator_len + output.offset();

    let mut directory_terminator = vec![0; directory_terminator_len];

    let directory_offset =
        u32::try_from(before_central_directory).expect("archive larger than 4GiB");
    let directory_count = u16::try_from(files.len()).expect("more than 64Ki files");
    let directory_length = u32::try_from(central_directory_len_without_terminator).unwrap();
    let suffix_length = u16::try_from(suffix.len()).expect("comment longer than 64KiB");

    // 0x0000..0x0004: archive terminator signature
    directory_terminator.write_all(b"PK\x05\x06").unwrap();
    // 0x0004..0x0006: disk number
    directory_terminator.write_all(&[0x00; 2]).unwrap();
    // 0x0006..0x0008: disk number with central directory
    directory_terminator.write_all(&[0x00; 2]).unwrap();
    // 0x0008..0x000A: directory entries on disk
    directory_terminator
        .write_all(&directory_count.to_le_bytes())
        .unwrap();
    // 0x000A..0x000C: directory entries total
    directory_terminator
        .write_all(&directory_count.to_le_bytes())
        .unwrap();
    // 0x000C..0x0010: central directory byte length
    directory_terminator
        .write_all(&directory_length.to_le_bytes())
        .unwrap();
    // 0x0010..0x0014: central directory offset from start of archive
    directory_terminator
        .write_all(&directory_offset.to_le_bytes())
        .unwrap();
    // 0x0014..: archive comment (suffix) length, then content
    directory_terminator
        .write_all(&suffix_length.to_le_bytes())
        .unwrap();

    *output.tagged("zip", "index-terminator") += &*directory_terminator;
    *output.tagged("zip", "comment") += suffix;
    let end = output.offset();

    Ok(end - start)
}

pub fn write_zip_archive_terminator() {}
