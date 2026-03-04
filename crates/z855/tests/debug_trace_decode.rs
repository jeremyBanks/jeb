use z855::z855::{encode, decode};

#[test]
fn repro_large_input() {
    // From proptest regression - the known failing input
    let data: Vec<u8> = vec![
        144, 221, 95, 52, 97, 55, 253, 24, 78, 14, 105, 236, 136, 97, 181, 174,
        117, 121, 25, 130, 95, 131, 56, 239, 24, 135, 215, 114, 133, 122, 184, 124,
        178, 87, 39, 115, 137, 38, 77, 81, 248, 128, 161, 246, 160, 62, 52, 206,
        139, 9, 214, 138, 222, 43, 97, 150, 197, 157, 99, 207, 72, 66, 57, 13,
        242, 100, 136, 145, 131, 231, 103, 133, 77, 52, 54, 8, 16, 182, 128, 34,
        83, 66, 61, 92, 215, 64, 139, 6, 189, 31, 143, 5, 189, 58, 100, 8,
        127, 78, 198, 99,
    ];

    // Try to find the minimal prefix that fails
    for len in 1..=data.len() {
        let sub = &data[..len];
        let enc = encode(sub);
        match decode(&enc) {
            Ok(dec) => {
                if dec != sub {
                    eprintln!("MISMATCH at len={}", len);
                    eprintln!("  encoded: {:?}", enc);
                    eprintln!("  expected {} bytes, got {} bytes", sub.len(), dec.len());
                    panic!("mismatch");
                }
            }
            Err(e) => {
                eprintln!("DECODE FAILED at len={}", len);
                eprintln!("  input bytes: {:?}", sub);
                eprintln!("  encoded ({} chars): {:?}", enc.len(), enc);
                eprintln!("  error: {:?}", e);

                // Also try decoding the previous length to confirm it works
                if len > 1 {
                    let prev = &data[..len-1];
                    let prev_enc = encode(prev);
                    match decode(&prev_enc) {
                        Ok(d) if d == prev => eprintln!("  len={} works fine", len-1),
                        _ => eprintln!("  len={} also fails!", len-1),
                    }
                }
                panic!("decode failed at len={}", len);
            }
        }
    }
    eprintln!("All prefixes up to {} bytes roundtrip OK", data.len());
}

#[test]
fn repro_large_input_full() {
    // Full 1000+ byte input from proptest - just test if ANY large random input fails
    let mut data = Vec::new();
    let mut rng = 42u64;
    for _ in 0..2000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((rng >> 33) as u8);
    }

    let encoded = encode(&data);
    match decode(&encoded) {
        Ok(decoded) => assert_eq!(decoded, data, "roundtrip mismatch"),
        Err(e) => {
            // Binary search for the minimal failing prefix
            let mut lo = 1usize;
            let mut hi = data.len();
            while lo < hi {
                let mid = (lo + hi) / 2;
                let sub = &data[..mid];
                let enc = encode(sub);
                match decode(&enc) {
                    Ok(d) if d == sub => lo = mid + 1,
                    _ => hi = mid,
                }
            }
            eprintln!("Minimum failing prefix: {} bytes", lo);
            let sub = &data[..lo];
            let enc = encode(sub);
            eprintln!("Encoded: {:?}", &enc[..enc.len().min(500)]);
            eprintln!("Error: {:?}", e);
            panic!("decode failed");
        }
    }
}
