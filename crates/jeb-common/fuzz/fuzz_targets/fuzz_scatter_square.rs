#![no_main]

use {
    jeb_common::bi::scatter_square::scatter_square,
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    // Test roundtrip and shell properties for scatter_square bijection

    // u16 -> (i8, i8) roundtrip
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        let (x, y): (i8, i8) = scatter_square(u);
        let back: u16 = scatter_square((x, y));
        assert_eq!(u, back, "roundtrip failed for u16 {} -> ({}, {})", u, x, y);
    }

    // (i8, i8) -> u16 roundtrip
    if data.len() >= 2 {
        let x = data[0] as i8;
        let y = data[1] as i8;
        let u: u16 = scatter_square((x, y));
        let (back_x, back_y): (i8, i8) = scatter_square(u);
        assert_eq!(
            (x, y),
            (back_x, back_y),
            "roundtrip failed for ({}, {})",
            x,
            y
        );
    }

    // u32 -> (i16, i16) roundtrip
    if data.len() >= 4 {
        let u = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let (x, y): (i16, i16) = scatter_square(u);
        let back: u32 = scatter_square((x, y));
        assert_eq!(u, back, "roundtrip failed for u32 {}", u);
    }

    // u64 -> (i32, i32) roundtrip
    if data.len() >= 8 {
        let u = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let (x, y): (i32, i32) = scatter_square(u);
        let back: u64 = scatter_square((x, y));
        assert_eq!(u, back, "roundtrip failed for u64 {}", u);
    }

    // Bijection: two different u16 inputs should produce different outputs
    if data.len() >= 4 {
        let a = u16::from_le_bytes([data[0], data[1]]);
        let b = u16::from_le_bytes([data[2], data[3]]);
        if a != b {
            let pair_a: (i8, i8) = scatter_square(a);
            let pair_b: (i8, i8) = scatter_square(b);
            assert_ne!(
                pair_a, pair_b,
                "different inputs {} and {} should produce different outputs",
                a, b
            );
        }
    }

    // Shell monotonicity in region A (first 65025 values)
    if data.len() >= 4 {
        let a = u16::from_le_bytes([data[0], data[1]]);
        let b = u16::from_le_bytes([data[2], data[3]]);
        let region_a_size = 65025u16;
        if a < region_a_size && b < region_a_size && a < b {
            let (x1, y1): (i8, i8) = scatter_square(a);
            let (x2, y2): (i8, i8) = scatter_square(b);
            let shell1 = (x1 as i32).abs().max((y1 as i32).abs());
            let shell2 = (x2 as i32).abs().max((y2 as i32).abs());
            assert!(
                shell2 >= shell1,
                "shell should be monotonic in region A: u={} shell={}, u={} shell={}",
                a,
                shell1,
                b,
                shell2
            );
        }
    }

    // Region B points: the last 511 points all involve i8::MIN
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        let region_a_size = 65025u16;
        if u >= region_a_size {
            let (x, y): (i8, i8) = scatter_square(u);
            assert!(
                x == i8::MIN || y == i8::MIN,
                "region B point at u={} is ({}, {}) which doesn't involve MIN",
                u,
                x,
                y
            );
        }
    }

    // Zero maps to origin
    {
        let (x, y): (i8, i8) = scatter_square(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }
});
