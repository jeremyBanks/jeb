fn main() {
    const Z85_CHARS: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    
    // Test encoding [50, 0, 0, 0]
    let padded = [50u8, 0, 0, 0];
    let mut value = 0u32;
    for &b in &padded {
        value = value * 256 + b as u32;
    }
    println!("Value from [50,0,0,0]: {}", value);
    
    // Encode to Z85
    let mut result = [0u8; 5];
    let mut v = value;
    for i in (0..5).rev() {
        result[i] = Z85_CHARS[(v % 85) as usize];
        v /= 85;
    }
    println!("Full Z85: {:?}", std::str::from_utf8(&result).unwrap());
    println!("Leading char: '{}' (byte {})", result[0] as char, result[0]);
    
    // Build decode table
    let mut table = [255u8; 256];
    for (i, &c) in Z85_CHARS.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    
    // Decode from [leading, '0', '0', '0', '0']
    let to_decode = [result[0], b'0', b'0', b'0', b'0'];
    let mut decoded_value = 0u32;
    for &c in &to_decode {
        let digit = table[c as usize];
        decoded_value = decoded_value * 85 + digit as u32;
    }
    println!("Decoded value: {}", decoded_value);
    
    // Extract bytes
    let mut bytes = [0u8; 4];
    for i in (0..4).rev() {
        bytes[i] = (decoded_value % 256) as u8;
        decoded_value /= 256;
    }
    println!("Decoded bytes: {:?}", bytes);
    println!("First byte: {} (expected 50)", bytes[0]);
}
