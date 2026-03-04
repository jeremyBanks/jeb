use z855::z855::{encode, decode};

#[test]
fn trace_block_alignment() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let seed = 81u64;
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let h = hasher.finish();
    let len = 1487usize;
    let mut data = Vec::with_capacity(len);
    let mut state = h;
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((state >> 33) as u8);
    }
    
    // Encode 1147 bytes (last length that decodes OK)
    let enc_1147 = encode(&data[..1147]);
    let dec_1147 = decode(&enc_1147).unwrap();
    assert_eq!(dec_1147.len(), 1147);
    assert_eq!(&dec_1147[..], &data[..1147]);
    
    // Encode 1148 bytes (first length that SHOULD work but we already know it does)
    let enc_1148 = encode(&data[..1148]);
    match decode(&enc_1148) {
        Ok(d) => {
            assert_eq!(&d[..], &data[..1148]);
            eprintln!("1148 bytes roundtrips OK");
        }
        Err(e) => {
            eprintln!("1148 bytes fails: {:?}", e);
        }
    }
    
    // Let me look at different encodings around the boundary
    for n in 1140..=1160 {
        if n > data.len() { break; }
        let enc = encode(&data[..n]);
        let ok = decode(&enc).is_ok();
        if !ok {
            eprintln!("FAIL: {} bytes -> {} chars, last 30: {:?}", n, enc.len(), 
                &enc[enc.len().saturating_sub(30)..]);
        }
    }
    
    // Now, let me find the pattern. I bet the issue is with a specific escape
    // that shifts block alignment. Let me check if removing the ; at pos 1168
    // (the last escape before the problematic ,) changes things.
    
    // Encode up to just before the ; at pos 1168
    // First, let me figure out how many input bytes correspond to 1168 output chars
    // Binary search
    let mut lo = 900usize;
    let mut hi = 1147usize;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if encode(&data[..mid]).len() <= 1168 {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    eprintln!("\n1168 output chars corresponds to ~{} input bytes", lo);
    eprintln!("Encode of {} bytes: {} chars", lo-1, encode(&data[..lo-1]).len());
    eprintln!("Encode of {} bytes: {} chars", lo, encode(&data[..lo]).len());
    
    // The encoded output has ; at pos 1168. What does the data look like there?
    eprintln!("\nData around byte {}:", lo);
    for i in lo.saturating_sub(5)..=(lo+10).min(data.len()-1) {
        let safe = data[i] >= 0x20 && data[i] <= 0x7e;
        eprintln!("  data[{}] = 0x{:02x} ({}){}", i, data[i], data[i],
            if safe { format!(" '{}'", data[i] as char) } else { String::new() });
    }
}
