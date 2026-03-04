use z855::z855::{encode, decode};

#[test]
fn debug_minimal_failing_input() {
    // Minimal failing input from proptest regression
    let data: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,127,35,35,33,35,33,35,35,40,35,127,0,0,0,0,0,0,0,0];
    let encoded = encode(&data);
    let z85_expected_len = ((data.len() + 3) / 4) * 5;
    // For 34 bytes: ceil(34/4)*5 = 9*5 = 45... wait, let me check
    // 34 / 4 = 8 remainder 2, so 8 full blocks + 1 partial
    // 8*5 + 3 = 43 characters
    let z85_len = (data.len() / 4) * 5 + if data.len() % 4 > 0 { data.len() % 4 + 1 } else { 0 };
    
    eprintln!("Input length: {}", data.len());
    eprintln!("Encoded: {:?}", encoded);
    eprintln!("Encoded length: {}", encoded.len());
    eprintln!("Z85 expected length: {}", z85_len);
    
    // Also try to decode
    match decode(&encoded) {
        Ok(decoded) => {
            eprintln!("Decoded length: {}", decoded.len());
            eprintln!("Roundtrip match: {}", decoded == data);
        }
        Err(e) => {
            eprintln!("Decode error: {:?}", e);
        }
    }
}
