use z855::z855::{encode, decode};

#[test]
fn debug_large_roundtrip_regression() {
    // First regression case from proptest
    let data: Vec<u8> = vec![32, 32, 32, 32, 39, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 34, 35, 35, 40, 32, 33, 35, 40, 33, 35, 35, 33, 35, 35, 40, 35, 34, 32, 32, 32, 32, 32, 32, 32, 35, 33, 35, 32, 33, 33, 35, 40, 35, 35, 35, 33, 35];
    
    eprintln!("Input length: {}", data.len());
    let encoded = encode(&data);
    eprintln!("Encoded: {:?}", encoded);
    eprintln!("Encoded length: {}", encoded.len());
    
    let z85_len = (data.len() / 4) * 5 + if data.len() % 4 > 0 { data.len() % 4 + 1 } else { 0 };
    eprintln!("Z85 expected length: {}", z85_len);
    
    match decode(&encoded) {
        Ok(decoded) => {
            eprintln!("Decoded length: {}", decoded.len());
            if decoded != data {
                eprintln!("ROUNDTRIP MISMATCH!");
                // Find first difference
                for i in 0..decoded.len().max(data.len()) {
                    let d = decoded.get(i).copied();
                    let o = data.get(i).copied();
                    if d != o {
                        eprintln!("  First diff at index {}: decoded={:?} original={:?}", i, d, o);
                        break;
                    }
                }
            } else {
                eprintln!("Roundtrip OK");
            }
        }
        Err(e) => {
            eprintln!("Decode error: {:?}", e);
        }
    }
}
