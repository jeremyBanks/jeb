use z855::z855::{encode, decode};

#[test]
fn debug_overflow_narrowed() {
    // Generate the same data as seed 81
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let seed = 81u64;
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let h = hasher.finish();
    let len = 1000 + (h % 1000) as usize;
    let mut data = Vec::with_capacity(len);
    let mut state = h;
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((state >> 33) as u8);
    }
    
    let encoded = encode(&data);
    let encoded_bytes = encoded.as_bytes();
    
    // Try to find the minimal prefix of the input that causes the overflow on decode
    // Binary search for the boundary
    for input_len in (1..=data.len()).rev() {
        let partial = &data[..input_len];
        let enc = encode(partial);
        match decode(&enc) {
            Ok(dec) => {
                if dec != partial {
                    eprintln!("MISMATCH at input_len={}", input_len);
                    break;
                }
            }
            Err(e) => {
                if input_len <= data.len() - 5 {
                    // Just keep searching down
                    continue;
                }
                eprintln!("Error at input_len={}: {:?}", input_len, e);
                eprintln!("Encoded: {:?}", &enc);
            }
        }
    }
    
    // Instead, try binary search
    let mut lo: usize = 1;
    let mut hi: usize = data.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let partial = &data[..mid];
        let enc = encode(partial);
        match decode(&enc) {
            Ok(dec) if dec == partial => {
                lo = mid + 1;
            }
            _ => {
                hi = mid;
            }
        }
    }
    
    // lo is the first failing length
    eprintln!("First failing input length: {}", lo);
    let partial = &data[..lo];
    let enc = encode(partial);
    eprintln!("Input ({} bytes): {:?}", lo, &partial[..lo.min(80)]);
    eprintln!("Encoded: {:?}", enc);
    match decode(&enc) {
        Ok(dec) => eprintln!("Decoded OK? Mismatch: {}", dec != partial),
        Err(e) => eprintln!("Error: {:?}", e),
    }
    
    // Also try lo-1
    if lo > 1 {
        let partial = &data[..lo-1];
        let enc = encode(partial);
        match decode(&enc) {
            Ok(dec) if dec == partial => eprintln!("input_len={} works fine", lo-1),
            Ok(_) => eprintln!("input_len={} decodes but mismatches", lo-1),
            Err(e) => eprintln!("input_len={} also fails: {:?}", lo-1, e),
        }
    }
}
