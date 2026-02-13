use z85_extended::{encode, decode};

fn main() {
    let mut data = vec![200u8];
    data.extend_from_slice(b"        ");
    
    println!("Input ({} bytes): {:?}", data.len(), data);
    
    let encoded = encode(&data);
    println!("Encoded ({} bytes): {:?}", encoded.len(), encoded);
    println!("Encoded (string): {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    println!("Decoded ({} bytes): {:?}", decoded.len(), decoded);
    
    if decoded == data {
        println!("✓ Roundtrip OK");
    } else {
        println!("✗ Roundtrip FAILED");
        println!("Expected: {:?}", data);
        println!("Got:      {:?}", decoded);
    }
}
