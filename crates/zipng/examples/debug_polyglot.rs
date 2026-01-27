//! Debug script to analyze polyglot PNG+ZIP byte structure

use std::fs;

fn main() {
    let path = "target/polyglot_test.png";
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read {}: {}", path, e);
            eprintln!("Run `cargo run --example polyglot_test` first");
            return;
        }
    };

    println!("=== POLYGLOT DEBUG ANALYSIS ===\n");
    println!("File size: {} bytes\n", data.len());

    // Find PNG structure
    println!("--- PNG STRUCTURE ---");
    if &data[0..8] == b"\x89PNG\r\n\x1A\n" {
        println!("PNG signature: OK");
    }

    // Parse chunks
    let mut pos = 8;
    let mut idat_start = 0;
    let mut idat_data_start = 0;
    let mut idat_len = 0;

    while pos + 8 <= data.len() {
        let len = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        let chunk_type = &data[pos+4..pos+8];
        let chunk_name = String::from_utf8_lossy(chunk_type);

        println!("Chunk at {}: {} ({} bytes)", pos, chunk_name, len);

        if chunk_type == b"IDAT" {
            idat_start = pos;
            idat_data_start = pos + 8;
            idat_len = len;
        }

        pos += 4 + 4 + len + 4;

        if chunk_type == b"IEND" {
            break;
        }
    }

    println!("\n--- IDAT INTERNAL STRUCTURE ---");
    println!("IDAT chunk starts at: {}", idat_start);
    println!("IDAT data starts at: {}", idat_data_start);
    println!("IDAT data length: {}", idat_len);

    let zlib_start = idat_data_start;
    println!("\nZlib header: {:02x} {:02x}", data[zlib_start], data[zlib_start + 1]);

    let deflate_start = zlib_start + 2;
    let block_header = data[deflate_start];
    let block_len = u16::from_le_bytes([data[deflate_start + 1], data[deflate_start + 2]]);
    let block_nlen = u16::from_le_bytes([data[deflate_start + 3], data[deflate_start + 4]]);

    println!("PNG's deflate block at {}: header={:02x}, LEN={}, NLEN={:04x}, valid={}",
             deflate_start, block_header, block_len, block_nlen, block_nlen == !block_len);

    let pixel_data_start = deflate_start + 5;
    println!("\nFiltered pixel data starts at file offset: {}", pixel_data_start);

    println!("\n--- ROW STRUCTURE (first 15 rows) ---");
    let row_size = 14; // 1 filter + 13 data
    for row in 0..15 {
        let row_start = pixel_data_start + row * row_size;
        if row_start + row_size > pixel_data_start + block_len as usize {
            break;
        }

        let filter = data[row_start];
        print!("Row {:2} @{:3}: f={:02x} | ", row, row_start, filter);
        for i in 1..row_size {
            let b = data[row_start + i];
            if b >= 0x20 && b < 0x7f {
                print!("{}", b as char);
            } else {
                print!(".");
            }
        }
        print!(" | ");
        for i in 1..row_size.min(8) {
            print!("{:02x} ", data[row_start + i]);
        }
        println!();
    }

    println!("\n--- ZIP LOCAL HEADERS ---");

    for i in 0..data.len().saturating_sub(4) {
        if &data[i..i+4] == b"PK\x03\x04" {
            println!("\n[Local header at offset {}]", i);

            let compression = u16::from_le_bytes([data[i+8], data[i+9]]);
            let comp_size = u32::from_le_bytes([data[i+18], data[i+19], data[i+20], data[i+21]]);
            let uncomp_size = u32::from_le_bytes([data[i+22], data[i+23], data[i+24], data[i+25]]);
            let name_len = u16::from_le_bytes([data[i+26], data[i+27]]);
            let extra_len = u16::from_le_bytes([data[i+28], data[i+29]]);

            let name_end = i + 30 + name_len as usize;
            let name = String::from_utf8_lossy(&data[i+30..name_end.min(data.len())]);

            println!("  Name: {:?}", name);
            println!("  Compression: {} (8=deflate)", compression);
            println!("  Sizes: compressed={}, uncompressed={}", comp_size, uncomp_size);
            println!("  Extra len: {}", extra_len);

            let content_start = name_end + extra_len as usize;
            println!("  Content at offset: {}", content_start);

            // Calculate which row the content starts in
            if content_start > pixel_data_start {
                let rel_offset = content_start - pixel_data_start;
                let row_num = rel_offset / row_size;
                let row_offset = rel_offset % row_size;
                println!("  Content in row {}, offset {} within row", row_num, row_offset);

                if row_offset == 0 {
                    println!("  ✓ Content starts at row boundary (filter byte position)");
                } else {
                    println!("  ✗ Content NOT at row boundary!");
                }
            }

            if compression == 8 && content_start + 5 < data.len() {
                println!("\n  Deflate stream analysis:");
                let mut dpos = content_start;
                let dend = (content_start + comp_size as usize).min(data.len());
                let mut block_num = 0;

                while dpos < dend && block_num < 8 {
                    if dpos >= data.len() { break; }

                    let hdr = data[dpos];
                    let bfinal = hdr & 1;
                    let btype = (hdr >> 1) & 3;

                    if btype == 0 && dpos + 5 <= data.len() {
                        let len = u16::from_le_bytes([data[dpos+1], data[dpos+2]]);
                        let nlen = u16::from_le_bytes([data[dpos+3], data[dpos+4]]);
                        let valid = nlen == !len;

                        // Show what row this block header is in
                        let rel = dpos.saturating_sub(pixel_data_start);
                        let in_row = rel / row_size;
                        let in_offset = rel % row_size;

                        println!("    Block {} @{} (row {} off {}): hdr={:02x} final={} LEN={} valid={}",
                                 block_num, dpos, in_row, in_offset, hdr, bfinal, len, valid);

                        if !valid {
                            println!("      ERROR: NLEN={:04x}, expected {:04x}", nlen, !len);
                        }

                        if bfinal == 1 {
                            println!("    ✓ Final block found");
                            break;
                        }

                        dpos += 5 + len as usize;
                    } else {
                        println!("    Block {} @{}: type={} (not stored block)", block_num, dpos, btype);
                        break;
                    }
                    block_num += 1;
                }

                if block_num >= 8 {
                    println!("    ... (truncated)");
                }
            }
        }
    }

    // Find EOCD
    for i in (0..data.len().saturating_sub(22)).rev() {
        if &data[i..i+4] == b"PK\x05\x06" {
            println!("\n--- EOCD at offset {} ---", i);
            let num_entries = u16::from_le_bytes([data[i+8], data[i+9]]);
            let cd_offset = u32::from_le_bytes([data[i+16], data[i+17], data[i+18], data[i+19]]);
            println!("  {} entries, CD at offset {}", num_entries, cd_offset);
            break;
        }
    }
}
