#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between signed and unsigned integer types which preserves the
ordering and distance relationships of values.
        "#
    };
}
use description;

impls! {
    i8: u8;
    i16: u16;
    i32: u32;
    i64: u64;
    i128: u128;
    isize: usize;
}

use crate::is;

#[doc = description!()]
pub fn signedness<T: Signedness>(value: T) -> T::Out {
    value.signedness()
}

#[doc = description!()]
pub trait Signedness {
    type Out;

    #[doc = description!()]
    fn signedness(self) -> Self::Out;
}

macro_rules! impls {
    {$( $signed:ident: $unsigned:ident; )+} => {$(
        impl Signedness for $signed {
            type Out = $unsigned;

            fn signedness(self) -> $unsigned {
                let high_mask = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self as $unsigned) ^ high_mask
            }
        }

        impl Signedness for $unsigned {
            type Out = $signed;

            fn signedness(self) -> $signed {
                let high_mask = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self ^ high_mask) as $signed
            }
        }
    )+}
}
use impls;

#[cfg(test)]
mod tests {
    use super::*;

    // Test roundtrip: signed -> unsigned -> signed
    #[test]
    fn roundtrip_i8_to_u8() {
        for i in i8::MIN..=i8::MAX {
            let u: u8 = signedness(i);
            let back: i8 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i8 {i}");
        }
    }

    #[test]
    fn roundtrip_u8_to_i8() {
        for u in u8::MIN..=u8::MAX {
            let i: i8 = signedness(u);
            let back: u8 = signedness(i);
            assert_eq!(u, back, "roundtrip failed for u8 {u}");
        }
    }

    // Test ordering preservation: signed order matches unsigned order
    #[test]
    fn order_preservation_i8() {
        let mut last_u: Option<u8> = None;
        for i in i8::MIN..=i8::MAX {
            let u: u8 = signedness(i);
            if let Some(prev) = last_u {
                assert!(
                    u > prev,
                    "order not preserved: i8 {i} -> u8 {u}, but previous was {prev}"
                );
            }
            last_u = Some(u);
        }
    }

    // Test distance preservation: |a - b| signed == |signedness(a) - signedness(b)|
    // unsigned
    #[test]
    fn distance_preservation_i8() {
        for a in i8::MIN..=i8::MAX {
            for b in i8::MIN..=i8::MAX {
                let signed_dist = (a as i32 - b as i32).unsigned_abs();
                let ua: u8 = signedness(a);
                let ub: u8 = signedness(b);
                let unsigned_dist = (ua as i32 - ub as i32).unsigned_abs();
                assert_eq!(
                    signed_dist, unsigned_dist,
                    "distance not preserved: {a} - {b} = {signed_dist}, but {ua} - {ub} = \
                     {unsigned_dist}"
                );
            }
        }
    }

    // Test bijection coverage for i8
    #[test]
    fn bijection_coverage_i8() {
        use std::collections::HashSet;
        let mut seen: HashSet<u8> = HashSet::new();
        for i in i8::MIN..=i8::MAX {
            let u: u8 = signedness(i);
            assert!(seen.insert(u), "duplicate output for i8 {i}: {u}");
        }
        assert_eq!(seen.len(), 256);
    }

    // Test specific values to verify the XOR high-bit mapping
    #[test]
    fn specific_values_i8() {
        // i8::MIN (-128) should map to u8 0
        assert_eq!(signedness(i8::MIN), 0u8);
        // -1 should map to 127
        assert_eq!(signedness(-1i8), 127u8);
        // 0 should map to 128
        assert_eq!(signedness(0i8), 128u8);
        // i8::MAX (127) should map to 255
        assert_eq!(signedness(i8::MAX), 255u8);
    }

    // Test roundtrip for i16 (full range)
    #[test]
    fn roundtrip_i16() {
        for i in i16::MIN..=i16::MAX {
            let u: u16 = signedness(i);
            let back: i16 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i16 {i}");
        }
    }

    // Test ordering for i16
    #[test]
    fn order_preservation_i16() {
        let mut last_u: Option<u16> = None;
        for i in i16::MIN..=i16::MAX {
            let u: u16 = signedness(i);
            if let Some(prev) = last_u {
                assert!(u > prev, "order not preserved at i16 {i}");
            }
            last_u = Some(u);
        }
    }

    // Test roundtrip for larger types (sampled)
    #[test]
    fn roundtrip_i32_sample() {
        let test_values: Vec<i32> = (i32::MIN..(i32::MIN + 1000))
            .chain((i32::MAX - 1000)..=i32::MAX)
            .chain(-1000..1000)
            .chain((0..10000).map(|i| i * 214748))
            .chain((0..10000).map(|i| i * -214748))
            .collect();

        for i in test_values {
            let u: u32 = signedness(i);
            let back: i32 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i32 {i}");
        }
    }

    #[test]
    fn roundtrip_i64_sample() {
        let test_values: Vec<i64> = (i64::MIN..(i64::MIN + 1000))
            .chain((i64::MAX - 1000)..=i64::MAX)
            .chain(-1000..1000)
            .collect();

        for i in test_values {
            let u: u64 = signedness(i);
            let back: i64 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i64 {i}");
        }
    }

    #[test]
    fn roundtrip_i128_sample() {
        let test_values: Vec<i128> = [i128::MIN, i128::MIN + 1, -1, 0, 1, i128::MAX - 1, i128::MAX]
            .into_iter()
            .collect();

        for i in test_values {
            let u: u128 = signedness(i);
            let back: i128 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i128 {i}");
        }
    }

    // Test that signedness maps boundaries correctly for all types
    #[test]
    fn boundary_mapping() {
        // i8
        assert_eq!(signedness(i8::MIN), u8::MIN);
        assert_eq!(signedness(i8::MAX), u8::MAX);
        // i16
        assert_eq!(signedness(i16::MIN), u16::MIN);
        assert_eq!(signedness(i16::MAX), u16::MAX);
        // i32
        assert_eq!(signedness(i32::MIN), u32::MIN);
        assert_eq!(signedness(i32::MAX), u32::MAX);
        // i64
        assert_eq!(signedness(i64::MIN), u64::MIN);
        assert_eq!(signedness(i64::MAX), u64::MAX);
        // i128
        assert_eq!(signedness(i128::MIN), u128::MIN);
        assert_eq!(signedness(i128::MAX), u128::MAX);
    }
}
