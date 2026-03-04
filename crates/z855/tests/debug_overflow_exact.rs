use z855::z855::{encode, decode};

fn gen_data(seed: u64, len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(len as u64);
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((state >> 33) as u8);
    }
    data
}

#[test]
fn exact_102_byte_case() {
    let data = gen_data(12, 102);
    eprintln!("Full data: {:?}", data);
    let enc = encode(&data);
    eprintln!("Encoded ({} chars): {:?}", enc.len(), enc);
    
    match decode(&enc) {
        Ok(d) => {
            assert_eq!(d, data, "mismatch");
            eprintln!("Roundtrip OK");
        }
        Err(e) => {
            eprintln!("Decode error: {:?}", e);
            
            // Find escapes
            for (i, &b) in enc.as_bytes().iter().enumerate() {
                match b {
                    b',' | b';' | b'_' | b'~' | b'|' => {
                        let start = if i >= 5 { i - 5 } else { 0 };
                        let end = (i + 15).min(enc.len());
                        eprintln!("  {} at pos {} context: {:?}", b as char, i,
                            &enc[start..end]);
                    }
                    _ => {}
                }
            }
            
            // Find where decode fails
            for end in (1..=enc.len()).rev() {
                if decode(&enc[..end]).is_ok() {
                    eprintln!("\nDecode OK at {} chars, FAIL at {} chars", end, end+1);
                    if end < enc.len() {
                        eprintln!("Failing char: '{}' (0x{:02x}) at pos {}", 
                            enc.as_bytes()[end] as char, enc.as_bytes()[end], end);
                        eprintln!("Context: {:?}", &enc[end.saturating_sub(10)..enc.len().min(end+10)]);
                    }
                    break;
                }
            }
            
            // Also try to shrink: find minimum prefix that fails
            for prefix_len in 1..=data.len() {
                let sub = &data[..prefix_len];
                let e = encode(sub);
                if decode(&e).is_err() {
                    eprintln!("\nMinimal failing prefix: {} bytes", prefix_len);
                    eprintln!("Data: {:?}", sub);
                    eprintln!("Encoded: {:?}", e);
                    break;
                }
            }
        }
    }
}
