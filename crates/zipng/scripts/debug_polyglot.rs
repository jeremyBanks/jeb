#!/usr/bin/env rust-script
//! Debug script to analyze polyglot PNG+ZIP byte structure
//! Run with: cargo run --example debug_polyglot
//! Or: rust-script scripts/debug_polyglot.rs

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
            idat_data_start = pos + 8; // After length and type
            idat_len = len;
        }

        pos += 4 + 4 + len + 4; // length + type + data + crc

        if chunk_type == b"IEND" {
            break;
        }
    }

    println!("\n--- IDAT INTERNAL STRUCTURE ---");
    println!("IDAT chunk starts at: {}", idat_start);
    println!("IDAT data starts at: {}", idat_data_start);
    println!("IDAT data length: {}", idat_len);

    // Parse zlib/deflate structure inside IDAT
    let zlib_start = idat_data_start;
    println!("\nZlib header: {:02x} {:02x}", data[zlib_start], data[zlib_start + 1]);

    // First deflate block header
    let deflate_start = zlib_start + 2;
    let block_header = data[deflate_start];
    let block_len = u16::from_le_bytes([data[deflate_start + 1], data[deflate_start + 2]]);
    let block_nlen = u16::from_le_bytes([data[deflate_start + 3], data[deflate_start + 4]]);

    println!("Deflate block at {}: header={:02x}, LEN={}, NLEN={:04x}, valid={}",
             deflate_start, block_header, block_len, block_nlen, block_nlen == !block_len);

    // Pixel data starts after deflate header
    let pixel_data_start = deflate_start + 5;
    println!("\nFiltered pixel data starts at: {}", pixel_data_start);

    // Show row structure (filter byte + 13 data bytes = 14 bytes per row)
    println!("\n--- ROW STRUCTURE (first 10 rows) ---");
    let row_size = 14; // 1 filter + 13 data
    for row in 0..10 {
        let row_start = pixel_data_start + row * row_size;
        if row_start + row_size > data.len() {
            break;
        }

        let filter = data[row_start];
        print!("Row {:2}: filter={:02x} | ", row, filter);
        for i in 1..row_size {
            let b = data[row_start + i];
            // Print printable ASCII or hex
            if b >= 0x20 && b < 0x7f {
                print!("{}", b as char);
            } else {
                print!("\\x{:02x}", b);
            }
        }
        println!();
    }

    // Find ZIP structures
    println!("\n--- ZIP STRUCTURES ---");

    // Find local headers
    for i in 0..data.len()-4 {
        if &data[i..i+4] == b"PK\x03\x04" {
            println!("\nLocal header at offset {}", i);

            let compression = u16::from_le_bytes([data[i+8], data[i+9]]);
            let crc = u32::from_le_bytes([data[i+14], data[i+15], data[i+16], data[i+17]]);
            let comp_size = u32::from_le_bytes([data[i+18], data[i+19], data[i+20], data[i+21]]);
            let uncomp_size = u32::from_le_bytes([data[i+22], data[i+23], data[i+24], data[i+25]]);
            let name_len = u16::from_le_bytes([data[i+26], data[i+27]]);
            let extra_len = u16::from_le_bytes([data[i+28], data[i+29]]);

            let name_end = i + 30 + name_len as usize;
            let name = String::from_utf8_lossy(&data[i+30..name_end]);

            println!("  Name: {:?} (len={})", name, name_len);
            println!("  Compression: {} (8=deflate, 0=stored)", compression);
            println!("  Compressed size: {}, Uncompressed: {}", comp_size, uncomp_size);
            println!("  Extra len: {}", extra_len);

            let content_start = name_end + extra_len as usize;
            println!("  Content starts at offset {}", content_start);

            // Analyze content as deflate
            if compression == 8 && content_start + 10 < data.len() {
                println!("  Content (first 20 bytes):");
                print!("    ");
                for j in 0..20.min(comp_size as usize) {
                    print!("{:02x} ", data[content_start + j]);
                }
                println!();

                // Try to interpret as deflate blocks
                println!("  Deflate interpretation:");
                let mut dpos = content_start;
                let dend = content_start + comp_size as usize;
                let mut block_num = 0;

                while dpos < dend && block_num < 10 {
                    let hdr = data[dpos];
                    let bfinal = hdr & 1;
                    let btype = (hdr >> 1) & 3;

                    if btype == 0 {
                        // Stored block
                        if dpos + 5 <= dend {
                            let len = u16::from_le_bytes([data[dpos+1], data[dpos+2]]);
                            let nlen = u16::from_le_bytes([data[dpos+3], data[dpos+4]]);
                            let valid = nlen == !len;
                            println!("    Block {}: STORED, final={}, LEN={}, valid={}",
                                     block_num, bfinal, len, valid);
                            if !valid {
                                println!("      NLEN mismatch! got {:04x}, expected {:04x}", nlen, !len);
                                break;
                            }
                            dpos += 5 + len as usize;
                        } else {
                            println!("    Block {}: truncated header", block_num);
                            break;
                        }
                    } else {
                        println!("    Block {}: type={} (not stored), unexpected!", block_num, btype);
                        break;
                    }

                    if bfinal == 1 {
                        println!("    (final block reached)");
                        break;
                    }
                    block_num += 1;
                }
            }
        }
    }

    // Find EOCD
    for i in (0..data.len()-22).rev() {
        if &data[i..i+4] == b"PK\x05\x06" {
            println!("\n--- EOCD at offset {} ---", i);
            let num_entries = u16::from_le_bytes([data[i+8], data[i+9]]);
            let cd_size = u32::from_le_bytes([data[i+12], data[i+13], data[i+14], data[i+15]]);
            let cd_offset = u32::from_le_bytes([data[i+16], data[i+17], data[i+18], data[i+19]]);
            println!("  {} entries, CD at offset {}, size {}", num_entries, cd_offset, cd_size);
            break;
        }
    }
}
