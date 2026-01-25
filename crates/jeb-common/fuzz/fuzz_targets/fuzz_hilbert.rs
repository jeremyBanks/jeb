#![no_main]

use jeb_common::bi::hilbert::hilbert;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test roundtrip and locality properties for Hilbert curve bijection

    // u16 -> (u8, u8) roundtrip
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        let (x, y): (u8, u8) = hilbert(u);
        let back: u16 = hilbert((x, y));
        assert_eq!(u, back, "roundtrip failed for u16 {}", u);
    }

    // (u8, u8) -> u16 roundtrip
    if data.len() >= 2 {
        let x = data[0];
        let y = data[1];
        let u: u16 = hilbert((x, y));
        let (back_x, back_y): (u8, u8) = hilbert(u);
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
        let (x, y): (u16, u16) = hilbert(u);
        let back: u32 = hilbert((x, y));
        assert_eq!(u, back, "roundtrip failed for u32 {}", u);
    }

    // u64 -> (u32, u32) roundtrip
    if data.len() >= 8 {
        let u = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let (x, y): (u32, u32) = hilbert(u);
        let back: u64 = hilbert((x, y));
        assert_eq!(u, back, "roundtrip failed for u64 {}", u);
    }

    // u128 -> (u64, u64) roundtrip
    if data.len() >= 16 {
        let u = u128::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let (x, y): (u64, u64) = hilbert(u);
        let back: u128 = hilbert((x, y));
        assert_eq!(u, back, "roundtrip failed for u128 {}", u);
    }

    // Locality property for u16: adjacent values have Manhattan distance 1
    if data.len() >= 2 {
        let u = u16::from_le_bytes([data[0], data[1]]);
        if u < u16::MAX {
            let (x1, y1): (u8, u8) = hilbert(u);
            let (x2, y2): (u8, u8) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan, 1,
                "adjacent values {} and {} should have Manhattan distance 1, got {}",
                u,
                u + 1,
                manhattan
            );
        }
    }

    // Locality property for u32: adjacent values have Manhattan distance 1
    if data.len() >= 4 {
        let u = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if u < u32::MAX {
            let (x1, y1): (u16, u16) = hilbert(u);
            let (x2, y2): (u16, u16) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan, 1,
                "adjacent values {} and {} should have Manhattan distance 1, got {}",
                u,
                u + 1,
                manhattan
            );
        }
    }

    // Bijection: two different u16 inputs should produce different outputs
    if data.len() >= 4 {
        let a = u16::from_le_bytes([data[0], data[1]]);
        let b = u16::from_le_bytes([data[2], data[3]]);
        if a != b {
            let pair_a: (u8, u8) = hilbert(a);
            let pair_b: (u8, u8) = hilbert(b);
            assert_ne!(
                pair_a, pair_b,
                "different inputs {} and {} should produce different outputs",
                a, b
            );
        }
    }

    // Distance bounds: Hilbert curve guarantees manhattan <= 3*sqrt(curve_dist)
    if data.len() >= 4 {
        let a = u16::from_le_bytes([data[0], data[1]]);
        let b = u16::from_le_bytes([data[2], data[3]]);
        let (x1, y1): (u8, u8) = hilbert(a);
        let (x2, y2): (u8, u8) = hilbert(b);
        let curve_dist = a.abs_diff(b) as f64;
        let manhattan = ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as f64;
        let bound = 3.0 * curve_dist.sqrt();
        assert!(
            manhattan <= bound + 1.0,
            "manhattan distance {} should be <= 3*sqrt({}) = {} for values {} and {}",
            manhattan,
            curve_dist,
            bound,
            a,
            b
        );
    }
});
