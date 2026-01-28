use std::fs;
use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or("target/samples/rust_projects.png".to_string());
    let data = fs::read(&path).unwrap();
    
    // Get row width from IHDR
    let mut row_width = 0u32;
    for i in 0..data.len()-8 {
        if &data[i..i+4] == b"IHDR" {
            row_width = u32::from_be_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]);
            break;
        }
    }
    println!("Row width: {} bytes", row_width);
    
    // Find ZIP local file headers
    let mut files = Vec::new();
    for i in 0..data.len()-4 {
        if &data[i..i+4] == b"PK\x03\x04" {
            let compressed_size = u32::from_le_bytes([data[i+18], data[i+19], data[i+20], data[i+21]]) as usize;
            let uncompressed_size = u32::from_le_bytes([data[i+22], data[i+23], data[i+24], data[i+25]]) as usize;
            let name_len = u16::from_le_bytes([data[i+26], data[i+27]]) as usize;
            let extra_len = u16::from_le_bytes([data[i+28], data[i+29]]) as usize;
            let name = String::from_utf8_lossy(&data[i+30..i+30+name_len]).to_string();
            
            let header_total = 30 + name_len + extra_len;
            let end_pos = i + header_total + compressed_size;
            
            files.push((i, end_pos, name, uncompressed_size, compressed_size, extra_len));
        }
    }
    
    println!("\nFiles with gaps (gap = position in row where header starts):");
    for (i, (start, end, name, _usize, csize, extra)) in files.iter().enumerate() {
        let gap_from_prev = if i > 0 { start - files[i-1].1 } else { *start };
        let row_offset = start % row_width as usize;
        let end_row_offset = end % row_width as usize;
        let visual_gap_rows = gap_from_prev / row_width as usize;
        
        println!("{:5} +{:5} ({:2} rows, end@{:3}): {} (extra={})", 
            start, gap_from_prev, visual_gap_rows, end_row_offset, name, extra);
    }
}
