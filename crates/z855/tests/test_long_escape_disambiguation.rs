#[cfg(test)]
mod test {
    /// Test round-trip correctness of `|` long escape at various positions.
    ///
    /// The before-block-disambiguation.md analysis says canonical minimum
    /// is needed for `|` escapes when K ≤ 16 and P > 0. This test tries
    /// to trigger such cases by placing 8+ safe bytes at various offsets.

    /// All printable ASCII safe chars that z855 recognizes.
    const SAFE: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    /// Non-safe bytes (high bytes, control chars) that force Z85 encoding.
    const UNSAFE: &[u8] = &[0x80, 0x90, 0xA0, 0xB0, 0xC0, 0xD0, 0xE0, 0xF0,
                             0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

    fn roundtrip(input: &[u8]) -> bool {
        let encoded = z855::encode(input);
        match z855::decode(&encoded) {
            Ok(decoded) => decoded == input,
            Err(_) => false,
        }
    }

    fn roundtrip_verbose(input: &[u8], label: &str) {
        let encoded = z855::encode(input);
        match z855::decode(&encoded) {
            Ok(decoded) => {
                if decoded != input {
                    let encoded_preview: String = encoded.chars().take(80).collect();
                    panic!(
                        "ROUND-TRIP MISMATCH [{}]\n  input ({} bytes): {:02x?}\n  encoded: {}\n  decoded ({} bytes): {:02x?}",
                        label, input.len(), &input[..input.len().min(32)],
                        encoded_preview, decoded.len(), &decoded[..decoded.len().min(32)]
                    );
                }
            }
            Err(e) => {
                let encoded_preview: String = encoded.chars().take(80).collect();
                panic!(
                    "DECODE ERROR [{}]: {:?}\n  input ({} bytes): {:02x?}\n  encoded: {}",
                    label, e, input.len(), &input[..input.len().min(32)], encoded_preview
                );
            }
        }
    }

    /// Test: N unsafe bytes followed by K safe bytes (K=8..20).
    /// The unsafe prefix shifts the safe run to different input positions,
    /// potentially causing the `|` escape at different output block positions.
    #[test]
    fn test_long_escape_after_unsafe_prefix() {
        for prefix_len in 0..=12 {
            for safe_len in 8..=20 {
                let mut input = Vec::new();
                // Unsafe prefix
                for i in 0..prefix_len {
                    input.push(UNSAFE[i % UNSAFE.len()]);
                }
                // Safe run
                for i in 0..safe_len {
                    input.push(SAFE[i % SAFE.len()]);
                }
                // Unsafe suffix to prevent rest-of-input optimization
                input.push(0xFF);
                input.push(0xFE);
                input.push(0xFD);
                input.push(0xFC);

                let label = format!("prefix={}, safe={}", prefix_len, safe_len);
                roundtrip_verbose(&input, &label);
            }
        }
    }

    /// Test: interleaved unsafe/safe blocks to force `|` at various block positions.
    #[test]
    fn test_long_escape_interleaved() {
        // Try every combination of 1-7 unsafe bytes, then 8-16 safe bytes, then 1-7 unsafe
        for pre in 1..=7 {
            for safe_len in 8..=16 {
                for post in 1..=7 {
                    let mut input = Vec::new();
                    for i in 0..pre {
                        input.push(UNSAFE[i % UNSAFE.len()]);
                    }
                    for i in 0..safe_len {
                        input.push(SAFE[i % SAFE.len()]);
                    }
                    for i in 0..post {
                        input.push(UNSAFE[i % UNSAFE.len()]);
                    }

                    let label = format!("pre={}, safe={}, post={}", pre, safe_len, post);
                    roundtrip_verbose(&input, &label);
                }
            }
        }
    }

    /// Test: multiple safe runs separated by single unsafe bytes.
    /// This tests consecutive `|` escapes that may leave block_pos in unexpected states.
    #[test]
    fn test_consecutive_long_escapes() {
        for gap in 1..=4 {
            for safe1 in 8..=12 {
                for safe2 in 8..=12 {
                    let mut input = Vec::new();
                    for i in 0..safe1 {
                        input.push(SAFE[i % SAFE.len()]);
                    }
                    for i in 0..gap {
                        input.push(UNSAFE[i % UNSAFE.len()]);
                    }
                    for i in 0..safe2 {
                        input.push(SAFE[(i + 5) % SAFE.len()]);
                    }
                    // Prevent rest-of-input
                    input.push(0xFF);

                    let label = format!("safe1={}, gap={}, safe2={}", safe1, gap, safe2);
                    roundtrip_verbose(&input, &label);
                }
            }
        }
    }

    /// Brute-force: all byte values at positions surrounding a safe run.
    /// For each possible byte value in the 4 bytes before a safe run,
    /// check round-trip correctness.
    #[test]
    fn test_long_escape_all_before_bytes() {
        let safe_run: Vec<u8> = (0..10).map(|i| SAFE[i % SAFE.len()]).collect();
        let suffix = vec![0xFF, 0xFE, 0xFD, 0xFC];

        for b0 in (0u8..=255).step_by(17) {
            for b1 in (0u8..=255).step_by(17) {
                for b2 in (0u8..=255).step_by(19) {
                    for b3 in (0u8..=255).step_by(23) {
                        let mut input = vec![b0, b1, b2, b3];
                        input.extend_from_slice(&safe_run);
                        input.extend_from_slice(&suffix);

                        if !roundtrip(&input) {
                            roundtrip_verbose(&input, &format!(
                                "before=[{:02x},{:02x},{:02x},{:02x}]", b0, b1, b2, b3
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Test with inputs that have exactly K=8 safe bytes (minimum for `|`)
    /// preceded by 1-3 bytes, checking all 256 values for the byte just before
    /// the safe run.
    #[test]
    fn test_long_escape_k8_boundary_byte() {
        let safe_run: Vec<u8> = (0..8).map(|i| SAFE[i % SAFE.len()]).collect();
        let suffix = vec![0xFF, 0xFE, 0xFD, 0xFC];

        for prefix_len in 1..=3 {
            for boundary_byte in 0u8..=255 {
                let mut input = Vec::new();
                // Fixed unsafe prefix (except last byte)
                for _ in 0..prefix_len - 1 {
                    input.push(0x80);
                }
                input.push(boundary_byte);
                input.extend_from_slice(&safe_run);
                input.extend_from_slice(&suffix);

                if !roundtrip(&input) {
                    roundtrip_verbose(&input, &format!(
                        "prefix_len={}, boundary={:02x}", prefix_len, boundary_byte
                    ));
                }
            }
        }
    }

    /// Test with very large K (1764+) to check if prefix length overflow
    /// causes block_pos to reach 5 before | is seen.
    #[test]
    fn test_long_escape_large_k() {
        let mut failures = Vec::new();
        for k in [1764, 1800, 2000, 5000, 10000, 50000, 65000] {
            let mut input = Vec::new();
            // 4 unsafe bytes before
            input.extend_from_slice(&[0x80, 0x90, 0xA0, 0xB0]);
            // K safe bytes
            for i in 0..k {
                input.push(SAFE[i % SAFE.len()]);
            }
            // 4 unsafe bytes after
            input.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]);

            if !roundtrip(&input) {
                let encoded = z855::encode(&input);
                let preview: String = encoded.chars().take(60).collect();
                eprintln!("FAIL large_k={}: encoded starts with: {}", k, preview);
                failures.push(k);
            }
        }
        assert!(failures.is_empty(), "Failed for K values: {:?}", failures);
    }

    /// Find the exact threshold where the bug kicks in.
    #[test]
    fn test_long_escape_find_threshold() {
        let mut first_fail = None;
        // Test K values around the boundary where 3-digit prefix starts (K=1764)
        for k in 1700..=1800 {
            let mut input = Vec::new();
            input.extend_from_slice(&[0x80, 0x90, 0xA0, 0xB0]);
            for i in 0..k {
                input.push(SAFE[i % SAFE.len()]);
            }
            input.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]);

            if !roundtrip(&input) {
                if first_fail.is_none() {
                    first_fail = Some(k);
                    let encoded = z855::encode(&input);
                    let prefix: String = encoded.chars().take(40).collect();
                    eprintln!("First failure at K={}: {}", k, prefix);
                }
            }
        }
        if let Some(k) = first_fail {
            panic!("Long escape round-trip fails starting at K={}", k);
        }
    }
}
