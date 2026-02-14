fn main() {
    // Test what happens with 65536 and 65537 bytes
    let input_65536: Vec<u8> = vec![b'a'; 65536];
    let input_65537: Vec<u8> = vec![b'a'; 65537];
    
    // We can't call encode/decode directly in main without linking to the lib
    // But we can see the structure
    
    // The issue is likely that encoding 65537 bytes produces invalid Z85
    // because the encoder caps at 65536, leaving 1 byte
    // But 1 byte can't be the start of a new long escape (min 8 bytes needed)
    
    println!("65536 bytes: should be encoded as 0|[65536 bytes]");
    println!("65537 bytes: caps raw_len at 65536, tries to encode remaining 1 byte");
    println!("            1 byte goes to standard Z85 path -> 2 Z85 chars");
    println!("            Total output: 0| + 65536 + 2 = 65540 chars");
    println!("            But the decoder might have issues with the split");
}
