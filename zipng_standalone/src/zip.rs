use std::io::Write;

use bstr::ByteSlice;

use crate::checksums::crc32;
use crate::padding::write_aligned_pad_end;
use crate::padding::write_aligned_pad_start;

const BLOCK_SIZE: usize = 1024;

pub fn zip<'files, Files>(files: Files) -> Vec<u8>
where Files: 'files + IntoIterator<Item = (&'files [u8], &'files [u8])> {
    let mut files: Vec<(&[u8], &[u8])> = files.into_iter().collect();
    files.sort_by_cached_key(|(path, body)| {
        (
            // file named "mimetype" goes first, for the sake of package formats including EPUB and
            // ODT.
            path != b"mimetype",
            // followed by any empty files, since they have no associated data and therefor weaker
            // alignment requirements, so we want to pack them all together.
            !body.is_empty(),
            // files before directories
            path.iter().filter(|&&b| b == b'/').count(),
            // then lexicographically by path
            *path,
            // and only then by body
            *body,
        )
    });
    zip_with(&files, Vec::new(), b"")
}

/// Creates a zip file from files in the order given, appending to the `prefix`
/// buffer `Vec` (which does not need to be empty), and ending with the
/// given `suffix`,
pub fn zip_with(files: &[(&[u8], &[u8])], prefix: Vec<u8>, suffix: &[u8]) -> Vec<u8> {
    let mut output = prefix;

    if suffix.find(b"PK\x05\x06").is_some() {
        panic!("Zip file suffix must not contain zip file terminator signature PK\\x05\\x06");
    }

    let mut files_with_offsets = Vec::with_capacity(files.len());

    for (name, body) in files.iter() {
        let mut header = Vec::new();
        // 0x0000..0x0004: local file header signature
        header.extend_from_slice(b"PK\x03\x04");
        // 0x0004..0x0006: version needed to extract
        header.extend_from_slice(&1_0_u16.to_le_bytes());
        // 0x0006..0x0008: general purpose bit flag
        header.extend_from_slice(&[0x00; 2]);
        // 0x0008..0x000A: compression method
        header.extend_from_slice(&[0x00; 2]);
        // 0x000A..0x000C: modification time
        header.extend_from_slice(b"PK");
        // 0x000C..0x000E: modification date
        header.extend_from_slice(b"PK");
        // 0x000E..0x0012: checksum
        header.extend_from_slice(&crc32(body).to_le_bytes());
        // 0x0012..0x0016: compressed size
        header.extend_from_slice(
            &u32::try_from(body.len())
                .expect("file size larger than 4GiB")
                .to_le_bytes(),
        );
        // 0x0016..0x001A: uncompressed size
        header.extend_from_slice(
            &u32::try_from(body.len())
                .expect("file size larger than 4GiB")
                .to_le_bytes(),
        );
        // 0x001A..0x001E: file name length
        header.extend_from_slice(
            &u16::try_from(name.len())
                .expect("file name larger than 64KiB")
                .to_le_bytes(),
        );
        // 0x001E..0x0022: extra fields length
        header.extend_from_slice(&[0x00; 2]);
        // 0x0022: file name, followed by extra fields (we have none)
        header.extend_from_slice(name);
        let range = if !body.is_empty() && name != b"mimetype" {
            let header_range = write_aligned_pad_start(&mut output, &header, BLOCK_SIZE);
            let body_range = write_aligned_pad_end(&mut output, body, BLOCK_SIZE);
            header_range.start..body_range.end
        } else {
            let before = output.len();
            output.extend_from_slice(&header);
            output.extend_from_slice(body);
            let after = output.len();
            before..after
        };
        files_with_offsets.push((*name, *body, range.start));
    }

    let mut central_directory = Vec::new();
    for (name, body, header_offset) in files_with_offsets.iter() {
        let name = name.to_vec();
        let name_length = u16::try_from(name.len()).expect("file name larger than 64KiB");
        let body_length = u32::try_from(body.len()).expect("file size larger than 4GiB");
        let header_offset = u32::try_from(*header_offset).expect("archive larger than 4GiB");
        let crc = crc32(body).to_le_bytes();
        let mut header = Vec::new();
        // 0x0000..0x0004: central file header signature
        header.extend_from_slice(b"PK\x01\x02");
        // 0x0004..0x0006: creator version and platform
        header.extend_from_slice(&1_0_u16.to_le_bytes());
        // 0x0006..0x0008: required version
        header.extend_from_slice(&1_0_u16.to_le_bytes());
        // 0x0008..0x000A: general purpose bit flag
        header.extend_from_slice(&[0x00; 2]);
        // 0x000A..0x000C: compression method
        header.extend_from_slice(&[0x00; 2]);
        // 0x000C..0x000E: modification time
        header.extend_from_slice(b"PK");
        // 0x000E..0x0010: modification date
        header.extend_from_slice(b"PK");
        // 0x0010..0x0014: checksum
        header.extend_from_slice(&crc);
        // 0x0014..0x0018: compressed size
        header.extend_from_slice(&body_length.to_le_bytes());
        // 0x0018..0x001C: uncompressed size
        header.extend_from_slice(&body_length.to_le_bytes());
        // 0x001C..0x001E: file name length
        header.extend_from_slice(&name_length.to_le_bytes());
        // 0x001E..0x0020: extra field length
        header.extend_from_slice(&[0x00; 2]);
        // 0x0020..0x0022: file comment length
        header.extend_from_slice(&[0x00; 2]);
        // 0x0022..0x0024: disk number
        header.extend_from_slice(&[0x00; 2]);
        // 0x0024..0x0026: internal file attributes
        header.extend_from_slice(&[0x00; 2]);
        // 0x0026..0x002A: external file attributes
        header.extend_from_slice(&[0x00; 4]);
        // 0x002A..0x002E: local file header offset from start of archive
        header.extend_from_slice(&header_offset.to_le_bytes());
        // 0x002E..: file name, followed by extra fields and comments (we have none)
        header.extend_from_slice(&name);

        central_directory.extend_from_slice(&header);
    }

    let archive_terminator = vec![0; 22 + suffix.len()];
    central_directory.extend_from_slice(&archive_terminator);

    let central_directory_range =
        write_aligned_pad_start(&mut output, &central_directory, BLOCK_SIZE);

    let final_len = output.len();
    let mut archive_terminator = &mut output[final_len - archive_terminator.len()..];

    let directory_offset =
        u32::try_from(central_directory_range.start).expect("archive larger than 4GiB");
    let directory_count = u16::try_from(files.len()).expect("more than 64Ki files");
    let directory_length =
        u32::try_from(central_directory_range.len() - archive_terminator.len()).unwrap();
    let suffix_length = u16::try_from(suffix.len()).expect("comment longer than 64KiB");

    // 0x0000..0x0004: archive terminator signature
    archive_terminator.write_all(b"PK\x05\x06").unwrap();
    // 0x0004..0x0006: disk number
    archive_terminator.write_all(&[0x00; 2]).unwrap();
    // 0x0006..0x0008: disk number with central directory
    archive_terminator.write_all(&[0x00; 2]).unwrap();
    // 0x0008..0x000A: directory entries on disk
    archive_terminator
        .write_all(&directory_count.to_le_bytes())
        .unwrap();
    // 0x000A..0x000C: directory entries total
    archive_terminator
        .write_all(&directory_count.to_le_bytes())
        .unwrap();
    // 0x000C..0x0010: central directory byte length
    archive_terminator
        .write_all(&directory_length.to_le_bytes())
        .unwrap();
    // 0x0010..0x0014: central directory offset from start of archive
    archive_terminator
        .write_all(&directory_offset.to_le_bytes())
        .unwrap();
    // 0x0014..: archive comment (suffix) length, then content
    archive_terminator
        .write_all(&suffix_length.to_le_bytes())
        .unwrap();
    archive_terminator.write_all(suffix).unwrap();
    output
}
