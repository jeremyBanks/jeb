#![no_main]

use jeb_common::bi::zig_zag::zig_zag;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test roundtrip properties for various integer sizes

    // i8/u8 roundtrip
    if let Some(&byte) = data.first() {
        let i = byte as i8;
        let u: u8 = zig_zag(i);
        let back: i8 = zig_zag(u);
        assert_eq!(i, back, "roundtrip failed for i8 {}", i);
    }

    // u8/i8 roundtrip
    if let Some(&byte) = data.first() {
        let i: i8 = zig_zag(byte);
        let back: u8 = zig_zag(i);
        assert_eq!(byte, back, "roundtrip failed for u8 {}", byte);
    }

    // i16/u16 roundtrip
    if data.len() >= 2 {
        let i = i16::from_le_bytes([data[0], data[1]]);
        let u: u16 = zig_zag(i);
        let back: i16 = zig_zag(u);
        assert_eq!(i, back, "roundtrip failed for i16 {}", i);
    }

    // i64/u64 roundtrip
    if data.len() >= 8 {
        let i = i64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let u: u64 = zig_zag(i);
        let back: i64 = zig_zag(u);
        assert_eq!(i, back, "roundtrip failed for i64 {}", i);
    }

    // i128/u128 roundtrip
    if data.len() >= 16 {
        let i = i128::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let u: u128 = zig_zag(i);
        let back: i128 = zig_zag(u);
        assert_eq!(i, back, "roundtrip failed for i128 {}", i);
    }

    // Magnitude ordering: for positive n and negative -(n+1), the negative maps lower
    if let Some(&byte) = data.first() {
        let n = (byte as i8).saturating_abs();
        if n > 0 && n < 127 {
            let neg = -n - 1;
            let pos = n;
            let u_neg: u8 = zig_zag(neg);
            let u_pos: u8 = zig_zag(pos);
            assert!(
                u_neg < u_pos,
                "magnitude ordering: {} should map before {}, got {} vs {}",
                neg,
                pos,
                u_neg,
                u_pos
            );
        }
    }
});
