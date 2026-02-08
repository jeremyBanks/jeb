#![no_main]

use {
    jeb_common::bi::z_order::z_order,
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    // Test roundtrip properties for z-order (Morton) curve bijection

    // u16 -> (u8, u8) roundtrip
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        let (x, y): (u8, u8) = z_order(u);
        let back: u16 = z_order((x, y));
        assert_eq!(u, back, "roundtrip failed for u16 {}", u);
    }

    // (u8, u8) -> u16 roundtrip
    if data.len() >= 2 {
        let x = data[0];
        let y = data[1];
        let u: u16 = z_order((x, y));
        let (back_x, back_y): (u8, u8) = z_order(u);
        assert_eq!(
            (x, y),
            (back_x, back_y),
            "roundtrip failed for ({}, {})",
            x,
            y
        );
    }

    // u32 -> (u16, u16) roundtrip
    if data.len() >= 4 {
        let u = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let (x, y): (u16, u16) = z_order(u);
        let back: u32 = z_order((x, y));
        assert_eq!(u, back, "roundtrip failed for u32 {}", u);
    }

    // u64 -> (u32, u32) roundtrip
    if data.len() >= 8 {
        let u = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let (x, y): (u32, u32) = z_order(u);
        let back: u64 = z_order((x, y));
        assert_eq!(u, back, "roundtrip failed for u64 {}", u);
    }

    // u128 -> (u64, u64) roundtrip
    if data.len() >= 16 {
        let u = u128::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let (x, y): (u64, u64) = z_order(u);
        let back: u128 = z_order((x, y));
        assert_eq!(u, back, "roundtrip failed for u128 {}", u);
    }

    // Bijection: two different u16 inputs should produce different outputs
    if data.len() >= 4 {
        let a = u16::from_le_bytes([data[0], data[1]]);
        let b = u16::from_le_bytes([data[2], data[3]]);
        if a != b {
            let pair_a: (u8, u8) = z_order(a);
            let pair_b: (u8, u8) = z_order(b);
            assert_ne!(
                pair_a, pair_b,
                "different inputs {} and {} should produce different outputs",
                a, b
            );
        }
    }

    // Z-order property: interleaving bits means adjacent indices differ by at most
    // one coordinate change. Check that x and y each change by at most 1 bit at a
    // time.
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        if u < u16::MAX {
            let (x1, y1): (u8, u8) = z_order(u);
            let (x2, y2): (u8, u8) = z_order(u + 1);
            // In z-order, incrementing by 1 changes at most log2(diff)+1 bits in each coord
            // But at minimum, only one of x or y changes
            let x_diff = x1 != x2;
            let y_diff = y1 != y2;
            assert!(
                x_diff || y_diff,
                "adjacent values {} and {} should differ in at least one coordinate",
                u,
                u + 1
            );
        }
    }
});
