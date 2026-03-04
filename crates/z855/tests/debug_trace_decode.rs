use z855::z855::{encode, decode};

#[test]
fn narrow_down_failure_region() {
    // Reproduce the exact failure
    let mut rng: u64 = 12345;

    // Find first failing trial
    let mut fail_data = Vec::new();
    for _trial in 0u64..1000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let size = 1000 + (rng as usize % 5000);
        let data: Vec<u8> = (0..size).map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng >> 33) as u8
        }).collect();

        let encoded = encode(&data);
        match decode(&encoded) {
            Ok(decoded) if decoded == data => {},
            _ => {
                fail_data = data;
                break;
            }
        }
    }
    assert!(!fail_data.is_empty(), "no failure found");

    // Binary search for minimum prefix that fails
    let mut lo = 1usize;
    let mut hi = fail_data.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let sub = &fail_data[..mid];
        let enc = encode(sub);
        match decode(&enc) {
            Ok(d) if d == sub => lo = mid + 1,
            _ => hi = mid,
        }
    }
    let min_len = lo;
    eprintln!("Minimum failing length: {}", min_len);

    // Now we know fail_data[..min_len] fails but fail_data[..min_len-1] works.
    // The two encodings differ because adding byte at min_len-1 changes the encoding.
    // Let's see WHERE in the encoding the change happens.

    let enc_ok = encode(&fail_data[..min_len-1]);
    let enc_fail = encode(&fail_data[..min_len]);

    eprintln!("OK encoding: {} chars", enc_ok.len());
    eprintln!("Fail encoding: {} chars", enc_fail.len());

    // Find where they diverge
    let common = enc_ok.as_bytes().iter().zip(enc_fail.as_bytes().iter())
        .take_while(|(a,b)| a==b).count();
    eprintln!("Diverge at char {}", common);

    // What's the encoded output around the divergence?
    let start = common.saturating_sub(20);
    let end = (common + 40).min(enc_ok.len()).min(enc_fail.len());
    eprintln!("OK  [{}..{}]: {:?}", start, end, &enc_ok[start..end.min(enc_ok.len())]);
    eprintln!("Fail[{}..{}]: {:?}", start, end, &enc_fail[start..end.min(enc_fail.len())]);

    // Now the key question: what's the decode error on enc_fail?
    // Let's progressively decode more of enc_fail to find where decode breaks
    let mut last_ok_len = 0;
    for end_pos in 1..=enc_fail.len() {
        let prefix = &enc_fail[..end_pos];
        match decode(&enc_fail[..end_pos]) {
            Ok(_) => last_ok_len = end_pos,
            Err(_) => {
                // Only print the first failure
                if last_ok_len == end_pos - 1 {
                    eprintln!("\nFirst decode failure at encoded position {}", end_pos);
                    let ctx_start = end_pos.saturating_sub(15);
                    eprintln!("Context: {:?}", &enc_fail[ctx_start..end_pos.min(enc_fail.len())]);

                    // What's at this position? Is it an escape?
                    let b = enc_fail.as_bytes()[end_pos - 1];
                    eprintln!("Byte at pos {}: {} ('{}')", end_pos-1, b, b as char);

                    // Can we decode up to end_pos-1?
                    let ok = decode(&enc_fail[..end_pos-1]);
                    eprintln!("Decode[..{}]: {:?}", end_pos-1, ok.as_ref().map(|v| v.len()).map_err(|e| format!("{:?}", e)));
                }
            }
        }
    }

    // Try a different approach: the full encoding fails to decode.
    // But does it fail because of the escapes, or because the output length is wrong?
    // Let's strip all escapes and just try to decode the Z85 portions.
    eprintln!("\nFull encoding ({} chars): last 30 chars = {:?}", enc_fail.len(), &enc_fail[enc_fail.len()-30..]);

    // Count total chars consumed by escape sequences
    let enc_bytes = enc_fail.as_bytes();
    let mut total_escape_chars = 0;
    let mut i = 0;
    while i < enc_bytes.len() {
        match enc_bytes[i] {
            b',' => { total_escape_chars += 5; i += 5; },
            b';' => { total_escape_chars += 6; i += 6; },
            b'_' => { total_escape_chars += 7; i += 7; },
            b'~' => { total_escape_chars += 8; i += 8; },
            b'|' => {
                eprintln!("Long escape at pos {}, context: {:?}", i, &enc_fail[i.saturating_sub(5)..enc_fail.len().min(i+20)]);
                break; // complex, skip
            }
            _ => { i += 1; }
        }
    }
    eprintln!("Total escape-consumed chars up to pos {}: {}", i, total_escape_chars);
    eprintln!("Remaining plain Z85 chars: {} (from {} total)", enc_bytes.len() - total_escape_chars, enc_bytes.len());

    panic!("Analysis complete");
}
