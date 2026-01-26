#![no_main]

use {
    jeb_common::bi::signedness::signedness,
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    // Test roundtrip properties for signedness bijection

    // i8/u8 roundtrip
    if let Some(&byte) = data.first() {
        let i = byte as i8;
        let u: u8 = signedness(i);
        let back: i8 = signedness(u);
        assert_eq!(i, back, "roundtrip failed for i8 {}", i);
    }

    // u8/i8 roundtrip
    if let Some(&byte) = data.first() {
        let i: i8 = signedness(byte);
        let back: u8 = signedness(i);
        assert_eq!(byte, back, "roundtrip failed for u8 {}", byte);
    }

    // i16/u16 roundtrip
    if data.len() >= 2 {
        let i = i16::from_le_bytes([data[0], data[1]]);
        let u: u16 = signedness(i);
        let back: i16 = signedness(u);
        assert_eq!(i, back, "roundtrip failed for i16 {}", i);
    }

    // i32/u32 roundtrip
    if data.len() >= 4 {
        let i = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let u: u32 = signedness(i);
        let back: i32 = signedness(u);
        assert_eq!(i, back, "roundtrip failed for i32 {}", i);
    }

    // i64/u64 roundtrip
    if data.len() >= 8 {
        let i = i64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let u: u64 = signedness(i);
        let back: i64 = signedness(u);
        assert_eq!(i, back, "roundtrip failed for i64 {}", i);
    }

    // i128/u128 roundtrip
    if data.len() >= 16 {
        let i = i128::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let u: u128 = signedness(i);
        let back: i128 = signedness(u);
        assert_eq!(i, back, "roundtrip failed for i128 {}", i);
    }

    // Order preservation for i8
    if data.len() >= 2 {
        let a = data[0] as i8;
        let b = data[1] as i8;
        let ua: u8 = signedness(a);
        let ub: u8 = signedness(b);
        assert_eq!(
            a.cmp(&b),
            ua.cmp(&ub),
            "order not preserved for {} vs {}",
            a,
            b
        );
    }

    // Order preservation for i64
    if data.len() >= 16 {
        let a = i64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let b = i64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let ua: u64 = signedness(a);
        let ub: u64 = signedness(b);
        assert_eq!(
            a.cmp(&b),
            ua.cmp(&ub),
            "order not preserved for {} vs {}",
            a,
            b
        );
    }

    // Distance preservation for i8
    if data.len() >= 2 {
        let a = data[0] as i8;
        let b = data[1] as i8;
        let signed_dist = (a as i32 - b as i32).unsigned_abs();
        let ua: u8 = signedness(a);
        let ub: u8 = signedness(b);
        let unsigned_dist = (ua as i32 - ub as i32).unsigned_abs();
        assert_eq!(
            signed_dist, unsigned_dist,
            "distance not preserved: {} - {} = {}, but {} - {} = {}",
            a, b, signed_dist, ua, ub, unsigned_dist
        );
    }
});
