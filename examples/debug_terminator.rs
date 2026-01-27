use zipng::polyglot::{build_polyglot, BitDepth, Luminance};

fn main() {
    let files = vec![(b"test.txt".as_ref(), b"Hello!".as_ref())];
    let result = build_polyglot(&files, 0, BitDepth::EightBit, Luminance, None);
    
    // Find the IDAT chunk
    let idat_pos = result.windows(4).position(|w| w == b"IDAT").unwrap();
    let idat_len = u32::from_be_bytes([result[idat_pos-4], result[idat_pos-3], result[idat_pos-2], result[idat_pos-1]]) as usize;
    
    println!("IDAT at position {}, length {}", idat_pos, idat_len);
    
    // Get IDAT data (after 4-byte type, 2 byte zlib header)
    let idat_start = idat_pos + 4 + 2;
    
    // Show first IDAT stored block (the one containing filtered PNG rows)
    println!("\nFirst IDAT stored block:");
    let block_header = &result[idat_start..idat_start+5];
    println!("  {:02X} {:02X} {:02X} {:02X} {:02X}", block_header[0], block_header[1], block_header[2], block_header[3], block_header[4]);
    println!("  BFINAL={}, BTYPE={}", block_header[0] & 1, (block_header[0] >> 1) & 3);
    let len = u16::from_le_bytes([block_header[1], block_header[2]]);
    let nlen = u16::from_le_bytes([block_header[3], block_header[4]]);
    println!("  LEN={}, NLEN={}", len, nlen);
    
    // Show the filtered PNG data (which contains the ZIP data)
    let filtered_start = idat_start + 5;
    println!("\nFiltered PNG rows (raw bytes, {} total):", len);
    for (i, chunk) in result[filtered_start..filtered_start + len as usize].chunks(69).enumerate() {
        print!("  Row {}: ", i);
        print!("[{:02X}] ", chunk[0]); // filter byte
        for b in &chunk[1..chunk.len().min(20)] {
            print!("{:02X} ", b);
        }
        if chunk.len() > 20 {
            print!("...");
        }
        println!();
    }
    
    // Find the ZIP local file header
    if let Some(pk_pos) = result.windows(4).position(|w| w == b"PK\x03\x04") {
        println!("\nZIP local file header at position {}:", pk_pos);
        println!("  Compressed size field at {}: {:02X} {:02X} {:02X} {:02X}", 
            pk_pos + 18,
            result[pk_pos + 18], result[pk_pos + 19], result[pk_pos + 20], result[pk_pos + 21]);
        let compressed_size = u32::from_le_bytes([result[pk_pos + 18], result[pk_pos + 19], result[pk_pos + 20], result[pk_pos + 21]]);
        println!("  Compressed size: {} bytes", compressed_size);
        
        // Find where compressed data starts
        let name_len = u16::from_le_bytes([result[pk_pos + 26], result[pk_pos + 27]]) as usize;
        let extra_len = u16::from_le_bytes([result[pk_pos + 28], result[pk_pos + 29]]) as usize;
        let data_start = pk_pos + 30 + name_len + extra_len;
        println!("  Data starts at: {} (name_len={}, extra_len={})", data_start, name_len, extra_len);
        
        // Show the first bytes of compressed data
        println!("\nCompressed data (first 32 bytes):");
        for i in 0..32.min(compressed_size as usize) {
            if i % 16 == 0 && i > 0 { println!(); }
            if i % 16 == 0 { print!("  "); }
            print!("{:02X} ", result[data_start + i]);
        }
        println!();
        
        // Try to interpret as DEFLATE stored blocks
        println!("\nDEFLATE block interpretation:");
        let mut pos = data_start;
        let mut block_num = 0;
        while pos < data_start + compressed_size as usize && block_num < 5 {
            let bfinal = result[pos] & 1;
            let btype = (result[pos] >> 1) & 3;
            println!("  Block {}: BFINAL={}, BTYPE={}", block_num, bfinal, btype);
            if btype == 0 {
                let block_len = u16::from_le_bytes([result[pos + 1], result[pos + 2]]);
                let block_nlen = u16::from_le_bytes([result[pos + 3], result[pos + 4]]);
                println!("    LEN={}, NLEN={} (expected {})", block_len, block_nlen, !block_len);
                if block_nlen != !block_len {
                    println!("    ERROR: NLEN mismatch!");
                }
                pos += 5 + block_len as usize;
            } else {
                break;
            }
            if bfinal == 1 { break; }
            block_num += 1;
        }
    }
}
