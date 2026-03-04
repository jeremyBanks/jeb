use z855::z855::{encode, decode};

#[test]
fn debug_overflow_case() {
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
    eprintln!("Input length: {}", data.len());
    eprintln!("Encoded length: {}", encoded.len());
    
    // Find where the decoder fails by trying to decode incrementally
    // First, try the full string
    match decode(&encoded) {
        Ok(_) => eprintln!("Full decode succeeded"),
        Err(e) => {
            eprintln!("Full decode error: {:?}", e);
            
            // Find the problematic region - scan for long escape sequences
            let bytes = encoded.as_bytes();
            for (i, &b) in bytes.iter().enumerate() {
                if b == b'|' {
                    eprintln!("Found | at position {}", i);
                    // Show context around it
                    let start = if i >= 10 { i - 10 } else { 0 };
                    let end = (i + 20).min(bytes.len());
                    eprintln!("  Context: {:?}", std::str::from_utf8(&bytes[start..end]).unwrap_or("(invalid utf8)"));
                }
            }
        }
    }
}
