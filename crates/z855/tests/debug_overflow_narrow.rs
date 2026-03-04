use z855::z855::{encode, decode};

#[test]
fn narrow_overflow() {
    let data: Vec<u8> = vec![145, 24, 24, 232, 12, 74, 31, 35, 40, 34, 91, 36, 170, 223, 186, 212, 13, 78, 66, 4, 146, 44, 68, 116, 53, 104, 90, 200, 222, 92, 204, 172, 154, 172, 18, 158, 56, 41, 80, 90, 134, 47, 183, 154, 177, 143, 31, 131, 239, 67, 221, 242, 16, 242, 227, 229, 182, 66, 3, 233, 190, 177, 54, 131, 234, 36, 60, 254, 47, 240, 147, 142, 218, 88, 191, 179, 56, 188, 250, 217, 222, 39, 159, 185, 67, 60, 126, 83, 119, 7, 204, 254, 7, 228, 252, 84, 132, 239, 194, 194, 129, 97];
    
    eprintln!("Input: {} bytes", data.len());
    let enc = encode(&data);
    eprintln!("Encoded ({} chars): {:?}", enc.len(), enc);
    
    // Find escapes
    for (i, &b) in enc.as_bytes().iter().enumerate() {
        match b {
            b',' | b';' | b'_' | b'~' | b'|' => {
                let start = if i >= 3 { i - 3 } else { 0 };
                let end = (i + 15).min(enc.len());
                eprintln!("  {} at pos {} context: {:?}", b as char, i,
                    std::str::from_utf8(&enc.as_bytes()[start..end]).unwrap_or("?"));
            }
            _ => {}
        }
    }
    
    // Binary search for the decode failure point
    for end in (enc.len()-20..=enc.len()).rev() {
        if end > enc.len() { continue; }
        match decode(&enc[..end]) {
            Ok(_) => {
                eprintln!("\nDecode of first {} chars OK", end);
                eprintln!("Decode of first {} chars FAILS", end+1);
                eprintln!("Char at pos {}: '{}' (0x{:02x})", end, enc.as_bytes()[end] as char, enc.as_bytes()[end]);
                break;
            }
            Err(_) => {}
        }
    }
    
    // Shrink: try removing bytes from the beginning
    for skip in 0..data.len() {
        let sub = &data[skip..];
        let e = encode(sub);
        match decode(&e) {
            Ok(d) if d == sub => {}
            Ok(_) => { eprintln!("Mismatch starting at byte {}", skip); break; }
            Err(err) => { 
                eprintln!("\nFails starting at byte {}: {:?}", skip, err);
                eprintln!("  data[{}..] first 20: {:?}", skip, &sub[..20.min(sub.len())]);
                eprintln!("  encoded: {:?}", &e[..80.min(e.len())]);
                if skip > 0 {
                    // Check skip-1
                    let sub2 = &data[skip-1..];
                    let e2 = encode(sub2);
                    match decode(&e2) {
                        Ok(d) if d == sub2 => eprintln!("  data[{}..] works", skip-1),
                        _ => {}
                    }
                }
            }
        }
    }
}
