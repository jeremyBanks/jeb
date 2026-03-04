use z855::z855::{encode, decode};

#[test]
fn find_failure_case() {
    // Try to find a large input that fails
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    for seed in 0u64..10000 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let h = hasher.finish();
        
        // Generate a pseudo-random vector of 1000-2000 bytes
        let len = 1000 + (h % 1000) as usize;
        let mut data = Vec::with_capacity(len);
        let mut state = h;
        for _ in 0..len {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            data.push((state >> 33) as u8);
        }
        
        let encoded = encode(&data);
        match decode(&encoded) {
            Ok(decoded) => {
                if decoded != data {
                    eprintln!("ROUNDTRIP MISMATCH at seed {}, len {}", seed, len);
                    panic!("roundtrip failed");
                }
            }
            Err(e) => {
                eprintln!("DECODE ERROR at seed {}, len {}: {:?}", seed, len, e);
                eprintln!("Input (first 100 bytes): {:?}", &data[..100.min(data.len())]);
                eprintln!("Encoded (first 200 chars): {:?}", &encoded[..200.min(encoded.len())]);
                panic!("decode failed");
            }
        }
    }
    eprintln!("All 10000 seeds passed");
}
