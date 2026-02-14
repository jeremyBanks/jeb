// Minimal test case for the Z85 length invariant bug.
//
// Bug: The extended Z85 encoder produces MORE characters than standard Z85
// for certain input patterns involving safe bytes followed by unsafe bytes.
//
// The bug triggers when:
// - N safe bytes are followed by unsafe bytes
// - N is NOT a multiple of 4
// - N >= 9 (the minimum for long passthrough)
//
// Root cause: The length-prefixed `|` escape does not maintain the Z85 length
// invariant when the safe byte count leaves a non-multiple-of-4 remainder.

use cleanroom::z85::{encode, encode_standard};

/// Check if the length invariant holds for a given input.
/// Returns Some((extended, standard, diff)) if invariant is violated.
fn check_length_invariant(input: &[u8]) -> Option<(String, String, i64)> {
    let extended = encode(input);
    let standard = encode_standard(input);
    let diff = extended.len() as i64 - standard.len() as i64;

    if diff != 0 {
        Some((extended, standard, diff))
    } else {
        None
    }
}

#[test]
fn test_minimal_length_bug() {
    // MINIMAL REPRODUCTION CASE:
    // 9 safe bytes + 1 unsafe byte
    let input: Vec<u8> = b"abcdefghi\x00".to_vec();

    let extended = encode(&input);
    let standard = encode_standard(&input);

    println!("MINIMAL LENGTH BUG REPRODUCTION:");
    println!("  Input: {:?} ({} bytes)", input, input.len());
    println!("  Input as string: {:?}", String::from_utf8_lossy(&input));
    println!("  Extended ({} chars): {}", extended.len(), extended);
    println!("  Standard ({} chars): {}", standard.len(), standard);
    println!("  Diff: {} chars", extended.len() as i64 - standard.len() as i64);

    // The bug: extended should NOT be longer than standard
    // (length invariant: extended.len() <= standard.len())
    assert!(
        extended.len() <= standard.len(),
        "Length invariant violated! Extended ({}) > Standard ({})",
        extended.len(),
        standard.len()
    );
}

#[test]
fn test_length_bug_variants() {
    println!("\nAll minimal bug cases (N safe + 1 unsafe where N is not multiple of 4):\n");

    // The bug occurs for N = 9, 10, 11, 13, 14, 15, 17, 18, 19, etc.
    // (N >= 9 for long passthrough, N % 4 != 0 for the bug)
    let mut bug_cases = Vec::new();

    for n in 8..=32 {
        let mut input: Vec<u8> = (0..n).map(|i| b'a' + (i as u8 % 26)).collect();
        input.push(0x00); // 1 unsafe byte

        let extended = encode(&input);
        let standard = encode_standard(&input);
        let diff = extended.len() as i64 - standard.len() as i64;

        if diff > 0 {
            bug_cases.push((n, input.clone(), extended.clone(), standard.clone(), diff));
            println!("{} safe + 1 unsafe: diff = +{}", n, diff);
            println!("  Extended: {}", extended);
            println!("  Standard: {}", standard);
            println!();
        }
    }

    assert!(
        bug_cases.is_empty(),
        "Found {} cases where extended encoding is longer than standard!",
        bug_cases.len()
    );
}

#[test]
fn test_exact_byte_sequence() {
    // The exact smallest byte sequence that triggers the bug:
    let input: [u8; 10] = [
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, // "abcdefghi" (9 safe bytes)
        0x00, // null byte (unsafe)
    ];

    let extended = encode(&input);
    let standard = encode_standard(&input);

    println!("\nEXACT BYTE SEQUENCE THAT TRIGGERS THE BUG:");
    println!("  Input bytes: {:?}", input);
    println!("  Extended ({} chars): {}", extended.len(), extended);
    println!("  Standard ({} chars): {}", standard.len(), standard);

    // This assertion will fail, demonstrating the bug
    assert_eq!(
        extended.len(),
        standard.len(),
        "Length invariant violated! Extended length {} != Standard length {}",
        extended.len(),
        standard.len()
    );
}

#[test]
fn test_accumulating_bug() {
    // Does the bug accumulate with multiple triggers?
    // Answer: No - each independent trigger adds at most +1 char total
    // (the encoding is smart enough to avoid repeated overhead)

    // Pattern: (4 unsafe + 9 safe + 1 unsafe) repeated
    for num_reps in [1, 10, 100, 419] {
        let mut input = Vec::new();
        for _ in 0..num_reps {
            input.extend(vec![0x00u8; 4]); // 4 unsafe to break passthrough chain
            input.extend(b"abcdefghi".iter().copied()); // 9 safe
            input.push(0x00); // 1 unsafe to trigger bug
        }

        let extended = encode(&input);
        let standard = encode_standard(&input);
        let diff = extended.len() as i64 - standard.len() as i64;

        println!("{} repetitions: {} bytes, diff = {} chars", num_reps, input.len(), diff);
    }

    // The bug gives +1 per trigger, but triggers don't accumulate simply
    // because the encoder optimizes across boundaries
}

#[test]
fn test_summarize_findings() {
    println!("\n========================================");
    println!("SUMMARY: Z85 Length Invariant Bug");
    println!("========================================\n");

    println!("SMALLEST INPUT THAT TRIGGERS THE BUG:");
    println!("  10 bytes: b\"abcdefghi\\x00\"");
    println!("  (9 safe ASCII bytes + 1 null byte)\n");

    println!("BUG PATTERN:");
    println!("  N safe bytes followed by unsafe bytes, where:");
    println!("  - N >= 9 (minimum for long passthrough)");
    println!("  - N % 4 != 0 (causes padding miscalculation)\n");

    println!("AFFECTED N VALUES:");
    println!("  9, 10, 11, 13, 14, 15, 17, 18, 19, 21, 22, 23, 25, 26, 27, ...\n");

    println!("NOT AFFECTED (N % 4 == 0):");
    println!("  8, 12, 16, 20, 24, 28, ...\n");

    println!("EXAMPLE (N=9):");
    let input = b"abcdefghi\x00";
    let extended = encode(input);
    let standard = encode_standard(input);
    println!("  Input: {:?}", input);
    println!("  Extended: {} ({} chars)", extended, extended.len());
    println!("  Standard: {} ({} chars)", standard, standard.len());
    println!("  Overhead: +1 char\n");

    println!("ROOT CAUSE:");
    println!("  The `|` escape for 9 safe bytes produces:");
    println!("    9|abcdefghi|00 (14 chars)");
    println!("  But standard Z85 for 10 bytes is only:");
    println!("    vpA.SwObN*3Zk (13 chars)");
    println!();
    println!("  The length-prefixed escape adds unnecessary overhead");
    println!("  when the safe byte count is not a multiple of 4.");
}
