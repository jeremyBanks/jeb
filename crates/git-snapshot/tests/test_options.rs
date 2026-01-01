use git_snapshot::{
    CommitIdStyle,
    SerializationOptions,
    parse,
    serialize,
};

#[test]
fn test_serialization_options() {
    // Load a simple fixture
    let input_yaml = std::fs::read_to_string("tests/fixtures/02-linear-history.int.in.yaml")
        .expect("fixture should exist");

    let repo = parse(&input_yaml).expect("should parse");

    // Test 1: Default options (dedup yes, short hashes yes, integer IDs)
    let output_default = serialize(&repo, CommitIdStyle::Integer, SerializationOptions::default());
    println!("=== DEFAULT (dedup=yes, short=yes, int) ===");
    println!("{}\n", output_default);

    // Test 2: No deduplication
    let output_no_dedup = serialize(&repo, CommitIdStyle::Integer, SerializationOptions {
        use_deduplication: false,
        ..SerializationOptions::default()
    });
    println!("=== NO DEDUP (dedup=no, short=yes, int) ===");
    println!("{}\n", output_no_dedup);

    // Test 3: Hex IDs with short hashes (default for hex)
    let output_hex_short = serialize(&repo, CommitIdStyle::Hex, SerializationOptions::default());
    println!("=== HEX SHORT (dedup=yes, short=yes, hex) ===");
    println!("{}\n", output_hex_short);

    // Test 4: Hex IDs with full hashes
    let output_hex_full = serialize(&repo, CommitIdStyle::Hex, SerializationOptions {
        force_full_hashes: true,
        ..SerializationOptions::default()
    });
    println!("=== HEX FULL (dedup=yes, full hashes, hex) ===");
    println!("{}\n", output_hex_full);

    // Test 5: Hex with no short hashes (same as force_full_hashes but without the flag)
    let output_hex_no_short = serialize(&repo, CommitIdStyle::Hex, SerializationOptions {
        use_short_hashes: false,
        ..SerializationOptions::default()
    });
    println!("=== HEX NO SHORT (dedup=yes, no short, hex) ===");
    println!("{}\n", output_hex_no_short);

    // Test 6: No dedup, full hashes
    let output_no_dedup_full = serialize(&repo, CommitIdStyle::Hex, SerializationOptions {
        use_deduplication: false,
        force_full_hashes: true,
        ..SerializationOptions::default()
    });
    println!("=== NO DEDUP + FULL HASH (dedup=no, full, hex) ===");
    println!("{}\n", output_no_dedup_full);

    // Verify all outputs can be round-tripped
    for (name, output, id_style, options) in [
        ("default", &output_default, CommitIdStyle::Integer, SerializationOptions::default()),
        ("no_dedup", &output_no_dedup, CommitIdStyle::Integer, SerializationOptions {
            use_deduplication: false,
            ..SerializationOptions::default()
        }),
        ("hex_short", &output_hex_short, CommitIdStyle::Hex, SerializationOptions::default()),
        ("hex_full", &output_hex_full, CommitIdStyle::Hex, SerializationOptions {
            force_full_hashes: true,
            ..SerializationOptions::default()
        }),
        ("hex_no_short", &output_hex_no_short, CommitIdStyle::Hex, SerializationOptions {
            use_short_hashes: false,
            ..SerializationOptions::default()
        }),
        ("no_dedup_full", &output_no_dedup_full, CommitIdStyle::Hex, SerializationOptions {
            use_deduplication: false,
            force_full_hashes: true,
            ..SerializationOptions::default()
        }),
    ] {
        let repo2 = parse(output).unwrap_or_else(|e| {
            panic!("Failed to parse {} output: {}", name, e)
        });

        // Re-serialize with same options should be stable
        let output2 = serialize(&repo2, id_style, options);

        assert_eq!(output, &output2, "Round-trip failed for {}", name);
        eprintln!("✓ Round-trip stable for {}", name);
    }
}
