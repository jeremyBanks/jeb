#![no_main]

use jeb_common::bi::floating::floating;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test roundtrip properties for floating point types

    // f32 roundtrip (via u32 bits)
    if data.len() >= 4 {
        let bits = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let f = f32::from_bits(bits);
        let encoded = floating(f);
        let decoded: f32 = floating(encoded);
        assert_eq!(
            f.to_bits(),
            decoded.to_bits(),
            "roundtrip failed for f32 with bits {:08x}",
            bits
        );
    }

    // f64 roundtrip (via u64 bits)
    if data.len() >= 8 {
        let bits = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let f = f64::from_bits(bits);
        let encoded = floating(f);
        let decoded: f64 = floating(encoded);
        assert_eq!(
            f.to_bits(),
            decoded.to_bits(),
            "roundtrip failed for f64 with bits {:016x}",
            bits
        );
    }

    // u32 roundtrip
    if data.len() >= 4 {
        let u = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let f: f32 = floating(u);
        let back: u32 = floating(f);
        assert_eq!(u, back, "roundtrip failed for u32 {}", u);
    }

    // u64 roundtrip
    if data.len() >= 8 {
        let u = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let f: f64 = floating(u);
        let back: u64 = floating(f);
        assert_eq!(u, back, "roundtrip failed for u64 {}", u);
    }

    // Order preservation for f32
    if data.len() >= 8 {
        let a_bits = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let b_bits = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let a = f32::from_bits(a_bits);
        let b = f32::from_bits(b_bits);
        let a_enc = floating(a);
        let b_enc = floating(b);
        assert_eq!(
            a.total_cmp(&b),
            a_enc.cmp(&b_enc),
            "order not preserved for f32 {:08x} vs {:08x}",
            a_bits,
            b_bits
        );
    }

    // Order preservation for f64
    if data.len() >= 16 {
        let a_bits = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let b_bits = u64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let a = f64::from_bits(a_bits);
        let b = f64::from_bits(b_bits);
        let a_enc = floating(a);
        let b_enc = floating(b);
        assert_eq!(
            a.total_cmp(&b),
            a_enc.cmp(&b_enc),
            "order not preserved for f64 {:016x} vs {:016x}",
            a_bits,
            b_bits
        );
    }

    // Bijection for f32: different bit patterns produce different encodings
    if data.len() >= 8 {
        let a_bits = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let b_bits = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if a_bits != b_bits {
            let a = f32::from_bits(a_bits);
            let b = f32::from_bits(b_bits);
            let a_enc = floating(a);
            let b_enc = floating(b);
            assert_ne!(
                a_enc, b_enc,
                "different f32 values should produce different encodings"
            );
        }
    }
});
