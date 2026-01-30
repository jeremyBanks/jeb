//! Debug the CD+EOCD alignment in polyglot output
use zipng::polyglot::{build_polyglot, BitDepth, ColorType};

fn main() {
    // Many small files - matching failing sample
    let file_data: Vec<(Vec<u8>, Vec<u8>)> = (0..50)
        .map(|i| {
            let name = format!("file_{i:03}.txt");
            let content = format!("Content of file {i}");
            (name.into_bytes(), content.into_bytes())
        })
        .collect();
    let files: Vec<(&[u8], &[u8])> = file_data.iter()
        .map(|(n, b)| (n.as_slice(), b.as_slice()))
        .collect();
    let result = build_polyglot(&files, 0, BitDepth::EightBit, ColorType::Luminance, None);

    // Parse IHDR to get dimensions
    let width = u32::from_be_bytes([result[16], result[17], result[18], result[19]]);
    let height = u32::from_be_bytes([result[20], result[21], result[22], result[23]]);
    let row_width = width as usize; // luminance 8bit = 1 bpp
    let frs = row_width + 1;
    println!("Image: {}x{}, row_width={}, frs={}", width, height, row_width, frs);
    println!("Total decompressed: {} (height*frs={})", height as usize * frs, height as usize * frs);

    // Find IDAT
    let mut pos = 8;
    let mut idat_data_start = 0;
    let mut idat_len = 0;
    while pos + 8 <= result.len() {
        let len = u32::from_be_bytes([result[pos], result[pos+1], result[pos+2], result[pos+3]]) as usize;
        let chunk_type = std::str::from_utf8(&result[pos+4..pos+8]).unwrap_or("????");
        println!("Chunk {} at offset {}, len={}", chunk_type, pos, len);
        if chunk_type == "IDAT" {
            idat_data_start = pos + 8;
            idat_len = len;
        }
        pos += 4 + 4 + len + 4;
        if chunk_type == "IEND" { break; }
    }

    // Decompress: walk stored deflate blocks
    let zlib_start = idat_data_start;
    println!("\nZlib header: {:02x} {:02x}", result[zlib_start], result[zlib_start+1]);
    let mut dpos = zlib_start + 2;
    let dend = idat_data_start + idat_len - 4; // minus adler32
    let mut decompressed = Vec::new();
    let mut block_num = 0;
    while dpos < dend {
        let hdr = result[dpos];
        let bfinal = hdr & 1;
        let btype = (hdr >> 1) & 3;
        if btype != 0 { println!("Block {}: non-stored (type {})", block_num, btype); break; }
        let len = u16::from_le_bytes([result[dpos+1], result[dpos+2]]) as usize;
        println!("Block {} at file offset {}: hdr={:02x} final={} len={}", block_num, dpos, hdr, bfinal, len);
        decompressed.extend_from_slice(&result[dpos+5..dpos+5+len]);
        dpos += 5 + len;
        block_num += 1;
        if bfinal == 1 { break; }
    }

    println!("\nDecompressed: {} bytes ({} rows of frs={})", decompressed.len(), decompressed.len() / frs, frs);

    // Show filter bytes and look for CD/EOCD
    let num_rows = decompressed.len() / frs;
    println!("\n--- Last 10 rows ---");
    let start_row = num_rows.saturating_sub(10);
    for row in start_row..num_rows {
        let row_start = row * frs;
        let filter = decompressed[row_start];
        let data_start = row_start + 1;
        let data_end = (row_start + frs).min(decompressed.len());
        let data = &decompressed[data_start..data_end];

        // Check for PK signatures in data
        let mut sig = "";
        if data.len() >= 4 {
            if &data[0..4] == b"PK\x01\x02" { sig = " <-- CD entry"; }
            if &data[0..4] == b"PK\x05\x06" { sig = " <-- EOCD"; }
        }

        print!("Row {:3}: filter={:02x}", row, filter);
        if filter != 0 { print!(" *** BAD FILTER ***"); }
        print!("{}", sig);
        print!(" | ");
        for i in 0..data.len().min(20) {
            print!("{:02x} ", data[i]);
        }
        println!();
    }

    // Also find CD/EOCD in decompressed stream
    println!("\n--- CD/EOCD locations in decompressed stream ---");
    for i in 0..decompressed.len().saturating_sub(4) {
        if &decompressed[i..i+4] == b"PK\x01\x02" {
            let row = i / frs;
            let off = i % frs;
            println!("CD entry at decompressed offset {} (row {}, offset {})", i, row, off);
        }
        if &decompressed[i..i+4] == b"PK\x05\x06" {
            let row = i / frs;
            let off = i % frs;
            println!("EOCD at decompressed offset {} (row {}, offset {})", i, row, off);
        }
    }

    // Parse CD entries from file (search near CD offset)
    println!("\n--- CD entries in file ---");
    for i in 0..result.len().saturating_sub(46) {
        if i + 4 <= result.len() && &result[i..i+4] == b"PK\x01\x02" {
            let name_len = u16::from_le_bytes([result[i+28], result[i+29]]) as usize;
            let extra_len = u16::from_le_bytes([result[i+30], result[i+31]]) as usize;
            let header_offset = u32::from_le_bytes([result[i+42], result[i+43], result[i+44], result[i+45]]);
            let name = String::from_utf8_lossy(&result[i+46..i+46+name_len]);
            println!("  CD entry at file {}: name={:?}, extra_len={}, header_offset={}", i, name, extra_len, header_offset);

            // Check what's at header_offset
            let ho = header_offset as usize;
            if ho + 4 <= result.len() {
                println!("    At header_offset {}: {:02x} {:02x} {:02x} {:02x}", ho, result[ho], result[ho+1], result[ho+2], result[ho+3]);
            }
        }
    }

    // Find EOCD in file and show its CD offset
    for i in (0..result.len().saturating_sub(22)).rev() {
        if &result[i..i+4] == b"PK\x05\x06" {
            let cd_offset_in_eocd = u32::from_le_bytes([result[i+16], result[i+17], result[i+18], result[i+19]]);
            let cd_size_in_eocd = u32::from_le_bytes([result[i+12], result[i+13], result[i+14], result[i+15]]);
            let num_entries = u16::from_le_bytes([result[i+8], result[i+9]]);
            println!("\nEOCD at file offset {}: {} entries, CD offset={}, CD size={}", i, num_entries, cd_offset_in_eocd, cd_size_in_eocd);

            // Check if CD signature exists at cd_offset
            let cdo = cd_offset_in_eocd as usize;
            if cdo + 4 <= result.len() {
                println!("Bytes at CD offset {}: {:02x} {:02x} {:02x} {:02x}", cdo, result[cdo], result[cdo+1], result[cdo+2], result[cdo+3]);
            }
            break;
        }
    }

    // Bytes after IEND
    let iend_pos = result.windows(4).position(|w| w == b"IEND").unwrap_or(0);
    let bytes_after_iend = result.len() - (iend_pos + 8);
    println!("Bytes after IEND: {}", bytes_after_iend);

    // Show bytes around computed CD offset
    println!("\n--- File bytes around CD offset 196661 ---");
    for off in [196600usize, 196661, 196731, 196796] {
        if off + 20 <= result.len() {
            print!("  offset {}: ", off);
            for i in 0..20 {
                print!("{:02x} ", result[off + i]);
            }
            // ASCII
            print!(" |");
            for i in 0..20 {
                let b = result[off + i];
                if b >= 0x20 && b < 0x7f { print!("{}", b as char); } else { print!("."); }
            }
            println!("|");
        }
    }

    // Try ZIP validation
    use std::io::Cursor;
    let cursor = Cursor::new(&result);
    match zip::ZipArchive::new(cursor) {
        Ok(mut archive) => {
            println!("\nZIP: {} files", archive.len());
            for i in 0..archive.len() {
                match archive.by_index(i) {
                    Ok(file) => println!("  {}: {} bytes", file.name(), file.size()),
                    Err(e) => println!("  file {}: ERROR {}", i, e),
                }
            }
        }
        Err(e) => println!("\nZIP error: {}", e),
    }
}
