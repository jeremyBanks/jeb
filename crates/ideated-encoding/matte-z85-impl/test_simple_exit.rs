use z85_extended::{encode, decode};

fn main() {
    // Simple test: 5 spaces (raw) + 3 zeros
    let data = b"     \x00\x00\x00";
    println!("Input: {:?}", data);
    
    let encoded = encode(data);
    println!("Encoded ({} bytes): {:?}", encoded.len(), encoded);
    println!("Encoded string: {:?}", String::from_utf8_lossy(&encoded));
    
    match decode(&encoded) {
        Ok(decoded) => {
            println!("Decoded: {:?}", decoded);
            assert_eq!(decoded, data);
            println!("✓ Roundtrip OK");
        }
        Err(e) => {
            println!("✗ Decode error: {:?}", e);
        }
    }
}
