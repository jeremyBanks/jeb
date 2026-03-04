use z855::z855::{encode, decode};

#[test]
fn isolate_the_overflow() {
    // Just test decoding the problematic Z85 block
    // #UJX= in Z85 should overflow u32
    let z85_alpha = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    let hash_idx = z85_alpha.iter().position(|&b| b == b'#').unwrap();
    eprintln!("# is Z85 index {}", hash_idx); // Should be 84
    
    // Check: max Z85 value for 5-char block
    eprintln!("85^4 = {}", 85u64.pow(4)); // 52200625
    eprintln!("84 * 85^4 = {}", 84u64 * 85u64.pow(4)); // 4384852500
    eprintln!("u32::MAX = {}", u32::MAX as u64); // 4294967295
    eprintln!("84 * 85^4 > u32::MAX: {}", 84u64 * 85u64.pow(4) > u32::MAX as u64);
    
    // So ANY 5-char Z85 block starting with # overflows.
    // The max valid first digit for a 5-char block is 82 ('%')
    // because 82 * 85^4 = 4280451250 < 4294967295
    // and 83 * 85^4 = 4332651875 > 4294967295 (nope: 83 * 52200625 = 4332651875 > u32::MAX)
    eprintln!("83 * 85^4 = {}", 83u64 * 85u64.pow(4));
    eprintln!("82 * 85^4 = {}", 82u64 * 85u64.pow(4));
    // So index 83 ('$') also overflows! Only index 82 ('%') works for first char.
    // Actually: max valid 5-char Z85 is the encoding of 0xFFFFFFFF.
    // 0xFFFFFFFF = 4294967295
    // 4294967295 / 85 = 50529027 rem 0
    // 50529027 / 85 = 594459 rem 12
    // 594459 / 85 = 6993 rem 54
    // 6993 / 85 = 82 rem 23
    // So max first digit is 82 ('%')
    // The max valid 5-char Z85 is "%nSc0" = 4294967295
    // So chars with index 83 ('$') or 84 ('#') are NEVER valid as the first char of a 5-char Z85 block.
    
    // Now the question: how does the encoder produce # as the first char of a Z85 block?
    // The answer is: it CAN'T. The # must be part of a passthrough, not a Z85 block.
    // But the DECODER is interpreting it as a Z85 block.
    
    // So either:
    // 1. The encoder is placing # at a Z85 block boundary (bug in encoder)
    // 2. The decoder is misaligned (bug in decoder)
    // 3. Both
    
    // Let me construct a simpler test case
    // The issue: after a , passthrough, the decoder continues reading Z85 blocks.
    // If the next bytes happen to have # as the first char of the block, overflow.
    
    // Simple case: 8 bytes where the first 4 are safe ASCII containing #
    let data: Vec<u8> = vec![0x23, 0x23, 0x23, 0x23, 0xFF, 0xFF, 0xFF, 0xFF];
    let enc = encode(&data);
    eprintln!("\nSimple test: {:?} -> {:?}", data, enc);
    match decode(&enc) {
        Ok(d) => eprintln!("Decoded OK: {:?}", d),
        Err(e) => eprintln!("Decode error: {:?}", e),
    }
    
    // Now: 4 non-safe bytes followed by 4 bytes that start with #
    let data2: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x23, 0x23, 0x23, 0x23];
    let enc2 = encode(&data2);
    eprintln!("Test 2: {:?} -> {:?}", data2, enc2);
    match decode(&enc2) {
        Ok(d) => eprintln!("Decoded OK: {:?}", d),
        Err(e) => eprintln!("Decode error: {:?}", e),
    }
    
    // Now: sequence that would produce # as Z85 first char followed by passthrough
    // For 4 bytes with first byte > 0xCE: Z85 starts with high-index chars
    // 0xCExxxxxx: 0xCE000000 = 3456106496 / 85^4 = 66.2 → first digit 66 = '='
    // 0xFFxxxxxxx: starts with 82 = '%'  
    // So # (84) can't appear as first Z85 char of a full block with valid data.
    // BUT it can appear after a passthrough if the block boundaries are wrong.
}
