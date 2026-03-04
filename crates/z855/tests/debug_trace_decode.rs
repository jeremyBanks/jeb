use z855::z855::{encode, decode};

fn get_failing_input() -> Vec<u8> {
    let mut rng: u64 = 12345;
    for _trial in 0u64..1000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let size = 1000 + (rng as usize % 5000);
        let data: Vec<u8> = (0..size).map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng >> 33) as u8
        }).collect();

        let encoded = encode(&data);
        if decode(&encoded).is_err() {
            return data;
        }
    }
    panic!("no failure found");
}

/// Simulate the decoder's actual logic including hash padding detection
fn trace_decoder_full(encoded: &str) -> (usize, bool) {
    let input = encoded.as_bytes();
    let mut in_idx = 0;
    let mut block_pos: usize = 0;
    let mut block_digits: usize = 0;
    let mut known_high: usize = 0;
    let mut needed: usize = 5;
    let mut hash_padding_triggered = false;

    while in_idx < input.len() {
        let byte = input[in_idx];

        // Check for hash padding (mirrors decoder at line 1658)
        if block_pos == 0 && (in_idx % 5) == 0 && byte == b'#' {
            let remaining = input.len() - in_idx;
            if remaining >= 5 {
                let mut hash_run = 0usize;
                while hash_run < 3 && input[in_idx + hash_run] == b'#' {
                    hash_run += 1;
                }
                if hash_run > 0 {
                    let num_chars = 5 - hash_run;
                    eprintln!("  !!! HASH PADDING at pos {} (hash_run={}, num_chars={}), known_high={}", in_idx, hash_run, num_chars, known_high);
                    hash_padding_triggered = true;
                    in_idx += 5;
                    block_digits = 0;
                    block_pos = 0;
                    known_high = 0;
                    needed = 5;
                    continue;
                }
            }
        }

        // Check for long escape
        if byte == b'|' {
            eprintln!("  pos {}: | (long escape), digits={}", in_idx, block_digits);
            break;
        }

        // Check for passthrough escape
        let pass_len = match byte {
            b',' => Some(4usize),
            b';' => Some(5),
            b'_' => Some(6),
            b'~' => Some(7),
            _ => None,
        };

        if let Some(pl) = pass_len {
            if pl == 4 {
                let p = block_pos;
                if p == 0 {
                    in_idx += 5;
                } else {
                    eprintln!("  pos {}: , (4-byte P={}), known_high={}", in_idx, p, known_high);
                    block_digits = 0;
                    block_pos = 0;
                    known_high = p;
                    needed = 5 - p;
                    in_idx += 5;
                }
            } else {
                // Extended passthrough
                if block_digits == 0 {
                    eprintln!("  pos {}: {} ({}-byte block-aligned)", in_idx, byte as char, pl);
                    in_idx += 1 + pl;
                } else {
                    let p = block_digits - 1;
                    eprintln!("  pos {}: {} ({}-byte P={}), digits={}, known_high={}", in_idx, byte as char, pl, p, block_digits, known_high);
                    block_digits = 0;
                    block_pos = 0;
                    known_high = 0;
                    needed = 5;
                    in_idx += 1 + pl;
                }
            }
        } else {
            // Regular Z85 digit
            block_digits += 1;
            block_pos += 1;
            in_idx += 1;
            if block_digits == needed {
                if known_high > 0 {
                    eprintln!("  pos {}: after-block complete (needed={}, known_high={})", in_idx, needed, known_high);
                }
                block_digits = 0;
                block_pos = 0;
                known_high = 0;
                needed = 5;
            }
        }
    }

    eprintln!("  Final: {} remaining digits at pos {}/{}, known_high={}", block_digits, in_idx, input.len(), known_high);
    (block_digits, hash_padding_triggered)
}

#[test]
fn trace_failing_decode() {
    let data = get_failing_input();

    for len in 3288..=3296 {
        if len > data.len() { continue; }
        let sub = &data[..len];
        let enc = encode(sub);
        let result = decode(&enc);
        match &result {
            Ok(d) if d == sub => {
                eprintln!("{} bytes: OK (enc_len={}, expected={})", len, enc.len(), z85_len(len));
            }
            Ok(_) => {
                eprintln!("{} bytes: MISMATCH", len);
            }
            Err(e) => {
                eprintln!("\n=== {} bytes: ERROR {:?} ===", len, e);
                eprintln!("  enc_len={}, expected={}", enc.len(), z85_len(len));
                let (remaining, hash_triggered) = trace_decoder_full(&enc);
                eprintln!("  Simulated remaining: {}, hash_padding_triggered: {}", remaining, hash_triggered);
            }
        }
    }

    panic!("Trace complete");
}

fn z85_len(n: usize) -> usize {
    (n * 5 + 3) / 4
}
