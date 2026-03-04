use z855::z855::{encode, decode};

#[test]
fn shrink_90_byte_case() {
    let data: Vec<u8> = vec![145, 24, 24, 232, 12, 74, 31, 35, 40, 34, 91, 36, 170, 223, 186, 212, 13, 78, 66, 4, 146, 44, 68, 116, 53, 104, 90, 200, 222, 92, 204, 172, 154, 172, 18, 158, 56, 41, 80, 90, 134, 47, 183, 154, 177, 143, 31, 131, 239, 67, 221, 242, 16, 242, 227, 229, 182, 66, 3, 233, 230, 106, 102, 97, 178, 152, 174, 90, 251, 114, 45, 4, 132, 192, 0, 133, 146, 224, 32, 25, 113, 77, 68, 93, 79, 174, 180, 83, 118, 230];
    
    let enc = encode(&data);
    eprintln!("90 bytes -> {:?}", enc);
    
    // Try to shrink from the start
    for start in 0..data.len() {
        let sub = &data[start..];
        let e = encode(sub);
        if decode(&e).is_err() {
            if start > 0 {
                eprintln!("Can remove first {} bytes", start);
                eprintln!("Minimal data: {:?}", sub);
                eprintln!("Encoded: {:?}", e);
            }
            break;
        }
    }
    
    // Try to remove individual bytes from the middle to minimize
    let mut best = data.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..best.len() {
            let mut test = best.clone();
            test.remove(i);
            let e = encode(&test);
            if decode(&e).is_err() {
                best = test;
                changed = true;
                break;
            }
        }
    }
    
    eprintln!("\nMinimal case ({} bytes): {:?}", best.len(), best);
    let enc = encode(&best);
    eprintln!("Encoded ({} chars): {:?}", enc.len(), enc);
    
    // Print escapes
    for (i, &b) in enc.as_bytes().iter().enumerate() {
        match b {
            b',' | b';' | b'_' | b'~' | b'|' => {
                let start = if i >= 5 { i - 5 } else { 0 };
                let end = (i + 15).min(enc.len());
                eprintln!("  {} at pos {}: {:?}", b as char, i, &enc[start..end]);
            }
            _ => {}
        }
    }
    
    // Find where decode breaks
    for end in (1..enc.len()).rev() {
        if decode(&enc[..end]).is_ok() {
            eprintln!("\nDecode OK at {} chars, FAIL at {} chars", end, end+1);
            eprintln!("Chars around failure: {:?}", &enc[end.saturating_sub(5)..enc.len().min(end+5)]);
            break;
        }
    }
}
