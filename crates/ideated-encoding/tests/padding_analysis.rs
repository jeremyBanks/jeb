//! Padding waste analysis for ideated-encoding.
//!
//! Measures overhead from padding characters emitted after raw passthrough
//! sequences. Compares actual encoded output against theoretical baselines.

use ideated_encoding::{PADDING_CHAR, decode, encode, is_safe_for_raw};

/// Analyze encoding output for a given input.
struct EncodingAnalysis {
    name: &'static str,
    input_len: usize,
    encoded_len: usize,
    padding_count: usize,
    safe_byte_count: usize,
    unsafe_byte_count: usize,
    pure_z85_len: usize,
    theoretical_min: usize,
}

impl EncodingAnalysis {
    fn new(name: &'static str, input: &[u8]) -> Self {
        let encoded = encode(input);

        // Verify roundtrip
        let decoded = decode(&encoded).unwrap();
        assert_eq!(
            decoded, input,
            "{}: roundtrip failed (decoded {} bytes, expected {})",
            name,
            decoded.len(),
            input.len()
        );

        // Count padding characters in output
        let padding_count = encoded.iter().filter(|&&b| b == PADDING_CHAR).count();

        // Count safe vs unsafe bytes in input
        let safe_byte_count = input.iter().filter(|&&b| is_safe_for_raw(b)).count();
        let unsafe_byte_count = input.len() - safe_byte_count;

        // Pure Z85 baseline: ceil(n * 5 / 4)
        let pure_z85_len = (input.len() * 5).div_ceil(4);

        // Theoretical minimum for this encoding:
        // - Safe bytes could pass through 1:1 (if contiguous and long enough)
        // - Unsafe bytes need Z85: ceil(n * 5 / 4)
        // - Plus escape overhead for transitions
        // This is a loose lower bound — actual minimum depends on byte ordering.
        // We'll use: safe_bytes * 1 + ceil(unsafe_bytes * 5 / 4) as a rough floor.
        let theoretical_min = if unsafe_byte_count == 0 {
            // All safe: ideal is 1:1 passthrough (plus some escape overhead)
            safe_byte_count
        } else if safe_byte_count == 0 {
            // All unsafe: pure Z85
            pure_z85_len
        } else {
            // Mixed: safe 1:1 + unsafe Z85
            safe_byte_count + (unsafe_byte_count * 5).div_ceil(4)
        };

        Self {
            name,
            input_len: input.len(),
            encoded_len: encoded.len(),
            padding_count,
            safe_byte_count,
            unsafe_byte_count,
            pure_z85_len,
            theoretical_min,
        }
    }

    fn overhead_vs_z85(&self) -> f64 {
        if self.pure_z85_len == 0 {
            return 0.0;
        }
        (self.encoded_len as f64 - self.pure_z85_len as f64) / self.pure_z85_len as f64 * 100.0
    }

    fn overhead_vs_input(&self) -> f64 {
        if self.input_len == 0 {
            return 0.0;
        }
        (self.encoded_len as f64 - self.input_len as f64) / self.input_len as f64 * 100.0
    }

    fn padding_pct_of_output(&self) -> f64 {
        if self.encoded_len == 0 {
            return 0.0;
        }
        self.padding_count as f64 / self.encoded_len as f64 * 100.0
    }

    fn z85_overhead_vs_input(&self) -> f64 {
        if self.input_len == 0 {
            return 0.0;
        }
        (self.pure_z85_len as f64 - self.input_len as f64) / self.input_len as f64 * 100.0
    }

    fn print_report(&self) {
        println!("### {}", self.name);
        println!("  Input:          {} bytes ({} safe, {} unsafe)",
            self.input_len, self.safe_byte_count, self.unsafe_byte_count);
        println!("  Safe ratio:     {:.1}%",
            self.safe_byte_count as f64 / self.input_len as f64 * 100.0);
        println!("  Encoded:        {} bytes", self.encoded_len);
        println!("  Pure Z85:       {} bytes (+{:.1}% over input)",
            self.pure_z85_len, self.z85_overhead_vs_input());
        println!("  Theoretical min: {} bytes (loose lower bound)", self.theoretical_min);
        println!("  Padding chars:  {} ({:.1}% of output)",
            self.padding_count, self.padding_pct_of_output());
        println!("  vs Z85:         {:+.1}%", self.overhead_vs_z85());
        println!("  vs input:       +{:.1}% overhead", self.overhead_vs_input());
        println!();
    }

    fn markdown_row(&self) -> String {
        format!(
            "| {} | {} | {:.0}% | {} | {} | {} | {:.1}% | {:+.1}% | +{:.1}% |",
            self.name,
            self.input_len,
            self.safe_byte_count as f64 / self.input_len as f64 * 100.0,
            self.encoded_len,
            self.pure_z85_len,
            self.padding_count,
            self.padding_pct_of_output(),
            self.overhead_vs_z85(),
            self.overhead_vs_input(),
        )
    }
}

// ── Sample data generators ──────────────────────────────────────────────────

fn sample_ascii_text() -> Vec<u8> {
    b"The quick brown fox jumps over the lazy dog. \
      Pack my box with five dozen liquor jugs. \
      How vexingly quick daft zebras jump! \
      The five boxing wizards jump quickly."
        .to_vec()
}

fn sample_json() -> Vec<u8> {
    br#"{"name":"Matte","type":"agent","model":"claude-opus-4-6","config":{"lookahead":64,"encoding":"extended-z85"},"tags":["encoder","binary","passthrough"],"metrics":{"overhead":0.25,"efficiency":0.80}}"#.to_vec()
}

fn sample_rust_source() -> Vec<u8> {
    br#"fn encode(data: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new();
    encoder.write(data);
    encoder.finish()
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let data = b"Hello, World!";
        let enc = encode(data);
        let dec = decode(&enc).unwrap();
        assert_eq!(dec, data);
    }
}"#
    .to_vec()
}

fn sample_mixed_binary_text() -> Vec<u8> {
    // Alternating: 16 bytes text, 16 bytes binary, repeated
    let mut data = Vec::new();
    for i in 0u8..8 {
        // 16 bytes of ASCII text
        let text = format!("chunk_{:02}_text!!", i);
        data.extend_from_slice(text.as_bytes());
        // 16 bytes of binary (high bytes, control chars, etc.)
        for j in 0u8..16 {
            data.push(i.wrapping_mul(31).wrapping_add(j.wrapping_mul(17)).wrapping_add(128));
        }
    }
    data
}

fn sample_pure_binary() -> Vec<u8> {
    // 256 bytes of pseudorandom binary data (many non-safe bytes)
    (0u8..=255)
        .map(|i| i.wrapping_mul(179).wrapping_add(83))
        .collect()
}

fn sample_mostly_safe() -> Vec<u8> {
    // Long string of safe-for-raw characters with occasional unsafe bytes
    let mut data = Vec::new();
    let safe_text = b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for i in 0..10 {
        data.extend_from_slice(safe_text);
        // Insert 2 unsafe bytes every 62 safe bytes
        data.push(0x00);
        data.push(0xFF - i);
    }
    data
}

fn sample_short_strings() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("1 byte safe", b"A".to_vec()),
        ("1 byte unsafe", vec![0x80]),
        ("4 bytes safe", b"test".to_vec()),
        ("4 bytes unsafe", vec![0x80, 0x90, 0xA0, 0xB0]),
        ("7 bytes safe", b"hello!!" .to_vec()),
        ("8 bytes safe", b"hello!!!" .to_vec()),
        ("16 bytes safe", b"0123456789abcdef".to_vec()),
        ("32 bytes safe", b"0123456789abcdef0123456789abcdef".to_vec()),
    ]
}

// ── Test ────────────────────────────────────────────────────────────────────

#[test]
fn padding_analysis() {
    println!("\n{}", "=".repeat(72));
    println!("  IDEATED-ENCODING PADDING WASTE ANALYSIS");
    println!("{}\n", "=".repeat(72));

    let mut analyses = Vec::new();

    // Main test cases
    let cases: Vec<(&'static str, Vec<u8>)> = vec![
        ("Pure ASCII text", sample_ascii_text()),
        ("JSON object", sample_json()),
        ("Rust source", sample_rust_source()),
        ("Mixed binary+text (50/50)", sample_mixed_binary_text()),
        ("Pure binary (256B)", sample_pure_binary()),
        ("Mostly safe (96% safe)", sample_mostly_safe()),
    ];

    println!("## Main test cases\n");
    for (name, data) in &cases {
        let a = EncodingAnalysis::new(name, data);
        a.print_report();
        analyses.push(a);
    }

    println!("\n## Short string behavior\n");
    for (name, data) in sample_short_strings() {
        let a = EncodingAnalysis::new(name, &data);
        a.print_report();
        analyses.push(a);
    }

    // Summary table
    println!("\n## Summary Table\n");
    println!("| Input | Bytes | Safe% | Encoded | Z85 | Pad | Pad% | vs Z85 | vs Input |");
    println!("|-------|-------|-------|---------|-----|-----|------|--------|----------|");
    for a in &analyses {
        println!("{}", a.markdown_row());
    }

    // Aggregate stats
    println!("\n## Aggregate\n");
    let total_input: usize = analyses.iter().map(|a| a.input_len).sum();
    let total_encoded: usize = analyses.iter().map(|a| a.encoded_len).sum();
    let total_z85: usize = analyses.iter().map(|a| a.pure_z85_len).sum();
    let total_padding: usize = analyses.iter().map(|a| a.padding_count).sum();
    println!("  Total input:    {} bytes", total_input);
    println!("  Total encoded:  {} bytes", total_encoded);
    println!("  Total pure Z85: {} bytes", total_z85);
    println!("  Total padding:  {} chars", total_padding);
    println!("  Overall vs Z85: {:+.1}%",
        (total_encoded as f64 - total_z85 as f64) / total_z85 as f64 * 100.0);
    println!("  Overall vs input: +{:.1}%",
        (total_encoded as f64 - total_input as f64) / total_input as f64 * 100.0);

    println!("\n  Padding as % of total output: {:.1}%",
        total_padding as f64 / total_encoded as f64 * 100.0);
}
