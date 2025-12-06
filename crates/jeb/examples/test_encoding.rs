// Test to verify new JEB85 encoding without block alignment

use jeb::encode_jeb85;

fn main() {
    // Test 1: Short ASCII text (< 4 bytes)
    let input1 = b"hi";
    let encoded1 = encode_jeb85(input1);
    println!("Input: {:?}", String::from_utf8_lossy(input1));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded1));
    println!("Length: {} bytes\n", encoded1.len());

    // Test 2: ASCII text (8 bytes, exactly 2 blocks)
    let input2 = b"12345678";
    let encoded2 = encode_jeb85(input2);
    println!("Input: {:?}", String::from_utf8_lossy(input2));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded2));
    println!("Length: {} bytes\n", encoded2.len());

    // Test 3: ASCII text (12 bytes, exactly 3 blocks)
    let input3 = b"Hello World!";
    let encoded3 = encode_jeb85(input3);
    println!("Input: {:?}", String::from_utf8_lossy(input3));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded3));
    println!("Length: {} bytes\n", encoded3.len());

    // Test 4: Mixed ASCII and binary
    let input4 = b"test\x00\x01\x02\x03more";
    let encoded4 = encode_jeb85(input4);
    println!("Input: {:?} (mixed ASCII and binary)", input4);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded4));
    println!("Length: {} bytes\n", encoded4.len());

    // Test 5: Verify no padding is present (old format would have dots)
    let input5 = b"abcdefgh"; // 8 bytes
    let encoded5 = encode_jeb85(input5);
    println!("Input: {:?}", String::from_utf8_lossy(input5));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded5));
    println!("Contains '.' (padding): {}", encoded5.contains(&b'.'));
    println!("Expected format: |<count>|<data>");
    println!("Length: {} bytes\n", encoded5.len());
}
