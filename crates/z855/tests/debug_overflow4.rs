use z855::z855::{encode, decode};

#[test]
fn find_minimal_overflow() {
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
    
    // 1152 works, 1153 fails. 
    // Encode 1153 bytes and find where the decoder chokes
    let data = &data[..1153];
    let encoded = encode(data);
    
    // The encoded string - scan it manually looking for long escape
    let bytes = encoded.as_bytes();
    eprintln!("Encoded length: {}", bytes.len());
    
    // Find all escape characters
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b',' => eprintln!("  , at pos {} (4-byte passthrough)", i),
            b';' => eprintln!("  ; at pos {} (5-byte passthrough)", i),
            b'_' => eprintln!("  _ at pos {} (6-byte passthrough)", i),
            b'~' => eprintln!("  ~ at pos {} (7-byte passthrough)", i),
            b'|' => {
                let start = if i >= 5 { i - 5 } else { 0 };
                let end = (i + 20).min(bytes.len());
                eprintln!("  | at pos {} (long escape) context: {:?}", i, 
                    std::str::from_utf8(&bytes[start..end]).unwrap_or("?"));
            },
            _ => {}
        }
    }
    
    // Try decoding progressively larger prefixes near the failure point
    // 1152 bytes encode correctly. 1152 * 5 / 4 = 1440 chars
    // Try decoding substrings of the encoded output
    let full_enc = &encoded;
    
    // Since 1152 bytes work, the encoded prefix for 1152 bytes should decode fine.
    // The issue is that when 1153 bytes are encoded, the encoder produces output
    // where the tail (after what would be the 1152-byte encoding) triggers a decode error.
    
    // Look at the end of the encoding
    let enc_len = encoded.len();
    eprintln!("\nLast 30 chars of encoding: {:?}", &encoded[enc_len-30..]);
    eprintln!("First 30 chars: {:?}", &encoded[..30]);
    
    // Try decode on the 1153-byte encoding string
    // Check char-by-char where the decoder would fail
    for end in (enc_len-20..=enc_len).rev() {
        match decode(&encoded[..end]) {
            Ok(_) => {
                eprintln!("Decode of first {} chars succeeds", end);
                break;
            }
            Err(e) => {
                eprintln!("Decode of first {} chars fails: {:?}", end, e);
            }
        }
    }
}
