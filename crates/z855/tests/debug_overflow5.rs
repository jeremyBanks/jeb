use z855::z855::{encode, decode};

#[test]
fn trace_encoder_output() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let seed = 81u64;
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let h = hasher.finish();
    let len = 1487usize;
    let mut data = Vec::with_capacity(len);
    let mut state = h;
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        data.push((state >> 33) as u8);
    }
    let data = &data[..1153];
    let encoded = encode(data);

    // Decode first 1434 chars successfully - that tells us the decoder state
    let ok_decoded = decode(&encoded[..1434]).unwrap();
    eprintln!("Decoded {} bytes from first 1434 chars", ok_decoded.len());
    
    // What's the input data around byte offset ~1147?
    let d = ok_decoded.len();
    eprintln!("Data around decoded end (bytes {}-{}):", d.saturating_sub(5), (d+10).min(data.len()));
    for i in d.saturating_sub(5)..(d+10).min(data.len()) {
        eprintln!("  data[{}] = {} (0x{:02x}) char={:?}", i, data[i], data[i], 
            if data[i] >= 0x20 && data[i] < 0x7f { data[i] as char } else { '.' });
    }
    
    // The encoded chars around position 1425-1442
    eprintln!("\nEncoded chars around 1420-1442:");
    let b = encoded.as_bytes();
    for i in 1420..encoded.len() {
        eprintln!("  pos[{}] = '{}' (0x{:02x})", i, b[i] as char, b[i]);
    }
    
    // How many output positions are accounted for by the decoded data?
    // The 4-byte passthrough at pos 1425 uses 5 chars (pos 1425-1429)
    // After the passthrough, the next block starts at pos 1430
    // The Z85 block at 1430-1434 is #UJX= which overflows
    
    // What byte offset does the decoder reach after the , passthrough?
    // The decoder successfully decoded 'ok_decoded.len()' bytes in 1434 chars.
    // With the , passthrough at 1425, the passthrough added 4 bytes to the output
    // and consumed 5 chars. So the decoder's block_pos should be 0 at 1430.
    
    // But if the encoder put # as the first char of a 5-char Z85 block, that
    // would always overflow. Let me check what the encoder intended.
    
    // Encode just the data up to ok_decoded.len()
    let enc_partial = encode(&data[..ok_decoded.len()]);
    eprintln!("\nEncode of first {} bytes: last 20 chars = {:?}", ok_decoded.len(), 
        &enc_partial[enc_partial.len().saturating_sub(20)..]);
    eprintln!("Length: {}", enc_partial.len());
    
    // Encode first ok_decoded.len()+1 bytes
    if ok_decoded.len() + 1 <= data.len() {
        let enc2 = encode(&data[..ok_decoded.len()+1]);
        eprintln!("Encode of first {} bytes: last 20 chars = {:?}", ok_decoded.len()+1,
            &enc2[enc2.len().saturating_sub(20)..]);
        eprintln!("Length: {}", enc2.len());
    }
}
