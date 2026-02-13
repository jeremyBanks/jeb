fn main() {
    const Z85_CHARS: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    
    // What do [50, 60, 0, 0] encode to?
    let bytes = [50u8, 60, 0, 0];
    let mut value = 0u32;
    for &b in &bytes {
        value = value * 256 + b as u32;
    }
    println!("Value from [50,60,0,0]: {}", value);
    
    let mut result = [0u8; 5];
    let mut v = value;
    for i in (0..5).rev() {
        result[i] = Z85_CHARS[(v % 85) as usize];
        v /= 85;
    }
    println!("Z85: {:?}", std::str::from_utf8(&result).unwrap());
    println!("First 2 chars: {}{}", result[0] as char, result[1] as char);
    
    // Now search for what bytes produce those first 2 chars
    let target = &result[0..2];
    println!("\nSearching for bytes that produce: {}{}", target[0] as char, target[1] as char);
    
    for b0 in 0..256u32 {
        for b1 in 0..256u32 {
            let test = [b0 as u8, b1 as u8, 0, 0];
            let mut v = 0u32;
            for &b in &test {
                v = v * 256 + b as u32;
            }
            let mut enc = [0u8; 5];
            let mut vv = v;
            for i in (0..5).rev() {
                enc[i] = Z85_CHARS[(vv % 85) as usize];
                vv /= 85;
            }
            if enc[0] == target[0] && enc[1] == target[1] {
                if b0 < 174 && b1 < 174 {
                    println!("  Found: [{}, {}] (both stable)", b0, b1);
                } else {
                    println!("  Found: [{}, {}] (NOT stable)", b0, b1);
                }
            }
        }
    }
}
