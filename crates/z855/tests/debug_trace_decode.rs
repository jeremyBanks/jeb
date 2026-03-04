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
fn verify_decode_manually() {
    let data = gen_data(12, 90);
    let enc = encode(&data);
    
    // Verify data matches what I expect
    assert_eq!(data[20], 146);
    assert_eq!(data[21], 44); // ','
    
    eprintln!("Data[20..30]: {:?}", &data[20..30]);
    
    // The encoded string
    eprintln!("Encoded: {:?}", enc);
    eprintln!("Encoded len: {}", enc.len());
    
    // Now let's verify: does the TypeScript encoder produce the same thing?
    // For now just check the Rust decoder on small substrings
    
    // Decode just up to position 99 (before the , escape)
    let prefix = &enc[..100];
    match decode(prefix) {
        Ok(d) => {
            eprintln!("First 100 chars decode to {} bytes", d.len());
            // How many bytes should 100 chars decode to?
            // 5 blocks (0-24) = 20 bytes
            // _ escape at 27 with passthrough 28-33 = 7 bytes (byte 20-26)
            // Blocks 34-98 = 13 blocks = 52 bytes (bytes 27-78)
            // Total: 20 + 7 + 52 = 79 bytes... does it match?
            assert_eq!(&d[..], &data[..d.len()], "prefix decode mismatch");
            eprintln!("Prefix decode matches first {} bytes of input", d.len());
        }
        Err(e) => eprintln!("First 100 chars decode error: {:?}", e),
    }
    
    // Decode up to position 105 (after the , escape + passthrough)
    let prefix2 = &enc[..109]; // through the after block
    match decode(prefix2) {
        Ok(d) => {
            eprintln!("First 109 chars decode to {} bytes", d.len());
            assert_eq!(&d[..], &data[..d.len()], "prefix2 decode mismatch");
        }
        Err(e) => eprintln!("First 109 chars decode error: {:?}", e),
    }
    
    // Decode the full string
    match decode(&enc) {
        Ok(d) => {
            eprintln!("Full decode to {} bytes", d.len());
            assert_eq!(&d[..], &data[..], "full decode mismatch");
        }
        Err(e) => {
            eprintln!("Full decode error: {:?}", e);
            // Try to find exactly where
            for end in 100..=enc.len() {
                match decode(&enc[..end]) {
                    Err(e) => {
                        if end <= 101 || decode(&enc[..end-1]).is_ok() {
                            eprintln!("  First failure at {} chars: {:?}", end, e);
                            eprintln!("  Chars {}-{}: {:?}", end.saturating_sub(5), end.min(enc.len()), &enc[end.saturating_sub(5)..end.min(enc.len())]);
                        }
                    }
                    Ok(d) => {
                        if end >= enc.len() - 1 {
                            eprintln!("  {} chars decode OK ({} bytes)", end, d.len());
                        }
                    }
                }
            }
        }
    }
}
