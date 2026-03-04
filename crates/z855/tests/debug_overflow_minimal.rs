use z855::z855::{encode, decode};

#[test]
fn find_small_overflow() {
    // Try many small inputs to find a case where encode produces output that fails to decode
    for len in 1..=500 {
        for seed in 0u64..100 {
            let mut data = Vec::with_capacity(len);
            let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(len as u64);
            for _ in 0..len {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                data.push((state >> 33) as u8);
            }
            
            let enc = encode(&data);
            match decode(&enc) {
                Ok(d) => {
                    if d != data {
                        eprintln!("MISMATCH at len={} seed={}", len, seed);
                        eprintln!("Data: {:?}", data);
                        panic!("mismatch");
                    }
                }
                Err(e) => {
                    eprintln!("DECODE FAIL at len={} seed={}: {:?}", len, seed, e);
                    eprintln!("Data: {:?}", &data[..data.len().min(60)]);
                    eprintln!("Encoded: {:?}", &enc[..enc.len().min(100)]);
                    panic!("decode fail");
                }
            }
        }
    }
    eprintln!("All passed for lengths 1-500");
}
