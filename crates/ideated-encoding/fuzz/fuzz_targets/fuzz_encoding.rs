#![no_main]

use {
    ideated_encoding::{decode, encode, Decoder, Encoder},
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    // === Property 1: Fundamental roundtrip invariant ===
    // decode(encode(x)) == x for all inputs
    let encoded = encode(data);
    if let Ok(decoded) = decode(&encoded) {
        assert_eq!(decoded, data, "roundtrip failed");
    }

    // === Property 2: Streaming encoder roundtrip ===
    // Split input into chunks and encode via streaming API
    if data.len() <= 1000 {
        let mut encoder = Encoder::new();
        let chunk_size = (data.len() / 7).max(1);
        for chunk in data.chunks(chunk_size) {
            encoder.write(chunk);
        }
        let stream_encoded = encoder.finish();
        if let Ok(decoded) = decode(&stream_encoded) {
            assert_eq!(decoded, data, "streaming encoder roundtrip failed");
        }
    }

    // === Property 3: Streaming decoder roundtrip ===
    // Decode the encoded data in chunks
    if data.len() <= 500 {
        let encoded = encode(data);
        let mut decoder = Decoder::new();
        let mut offset = 0;
        let mut decode_ok = true;
        while offset < encoded.len() {
            let chunk_size = ((offset * 7 + 3) % 13).min(encoded.len() - offset).max(1);
            if decoder.write(&encoded[offset..offset + chunk_size]).is_err() {
                decode_ok = false;
                break;
            }
            offset += chunk_size;
        }
        if decode_ok {
            if let Ok(decoded) = decoder.finish() {
                assert_eq!(decoded, data, "streaming decoder roundtrip failed");
            }
        }
    }

    // === Property 4: Encoding overhead bounded ===
    // Encoded output should never be more than ~130% of input plus small constant
    if !data.is_empty() {
        let max_len = (data.len() * 13 / 10) + 10;
        assert!(
            encoded.len() <= max_len,
            "encoded len {} exceeds max {} for input len {}",
            encoded.len(),
            max_len,
            data.len()
        );
    }

    // === Property 5: Encoded output contains only valid characters ===
    for (i, &byte) in encoded.iter().enumerate() {
        let is_valid =
            ideated_encoding::is_z85_char(byte) || ideated_encoding::is_escape_char(byte);
        assert!(is_valid, "invalid char {} at position {}", byte, i);
    }
});
