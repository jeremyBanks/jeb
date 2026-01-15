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
    i8 : u8; i16 : u16; i32 : u32; i64 : u64; i128 : u128; isize : usize;
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
    {$($signed:ident : $unsigned:ident;)+} => {
        $(impl Signedness for $signed { type Out = $unsigned; fn signedness(self) ->
        $unsigned { let high_mask = (is::<$unsigned > (1) << ($unsigned ::BITS - 1));
        (self as $unsigned) ^ high_mask } } impl Signedness for $unsigned { type Out =
        $signed; fn signedness(self) -> $signed { let high_mask = (is::<$unsigned > (1)
        << ($unsigned ::BITS - 1)); (self ^ high_mask) as $signed } })+
    };
}
use impls;
#[cfg(test)]
mod tests {
    use {
        super::*,
        proptest::prelude::*,
    };

    // ========================
    // Property-Based Tests
    // ========================
    //
    // These tests use proptest to verify properties hold for randomly generated
    // values. Properties tested:
    // 1. Roundtrip: signedness(signedness(x)) == x
    // 2. Order preservation: if a < b then signedness(a) < signedness(b)
    // 3. Distance preservation: |a - b| == |signedness(a) - signedness(b)|

    proptest! {
        /// Roundtrip property: encoding and decoding returns the original i8 value
        #[test]
        fn prop_roundtrip_i8(i in i8::MIN..=i8::MAX) {
            let u: u8 = signedness(i);
            let back: i8 = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for i8 {}", i);
        }

        /// Roundtrip property: decoding and encoding returns the original u8 value
        #[test]
        fn prop_roundtrip_u8(u in u8::MIN..=u8::MAX) {
            let i: i8 = signedness(u);
            let back: u8 = signedness(i);
            prop_assert_eq!(u, back, "roundtrip failed for u8 {}", u);
        }

        /// Roundtrip property for i16
        #[test]
        fn prop_roundtrip_i16(i in i16::MIN..=i16::MAX) {
            let u: u16 = signedness(i);
            let back: i16 = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for i16 {}", i);
        }

        /// Roundtrip property for i32
        #[test]
        fn prop_roundtrip_i32(i in proptest::num::i32::ANY) {
            let u: u32 = signedness(i);
            let back: i32 = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for i32 {}", i);
        }

        /// Roundtrip property for i64
        #[test]
        fn prop_roundtrip_i64(i in proptest::num::i64::ANY) {
            let u: u64 = signedness(i);
            let back: i64 = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for i64 {}", i);
        }

        /// Roundtrip property for i128
        #[test]
        fn prop_roundtrip_i128(i in proptest::num::i128::ANY) {
            let u: u128 = signedness(i);
            let back: i128 = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for i128 {}", i);
        }

        /// Roundtrip property for isize
        #[test]
        fn prop_roundtrip_isize(i in proptest::num::isize::ANY) {
            let u: usize = signedness(i);
            let back: isize = signedness(u);
            prop_assert_eq!(i, back, "roundtrip failed for isize {}", i);
        }

        /// Order preservation: if a < b then signedness(a) < signedness(b)
        #[test]
        fn prop_order_preservation_i8(a in i8::MIN..=i8::MAX, b in i8::MIN..=i8::MAX) {
            let ua: u8 = signedness(a);
            let ub: u8 = signedness(b);
            prop_assert_eq!(a.cmp(&b), ua.cmp(&ub), "order not preserved for {} vs {}", a, b);
        }

        /// Order preservation for i64
        #[test]
        fn prop_order_preservation_i64(a in proptest::num::i64::ANY, b in proptest::num::i64::ANY) {
            let ua: u64 = signedness(a);
            let ub: u64 = signedness(b);
            prop_assert_eq!(a.cmp(&b), ua.cmp(&ub), "order not preserved for {} vs {}", a, b);
        }

        /// Distance preservation: |a - b| == |signedness(a) - signedness(b)|
        #[test]
        fn prop_distance_preservation_i8(a in i8::MIN..=i8::MAX, b in i8::MIN..=i8::MAX) {
            let signed_dist = (a as i32 - b as i32).unsigned_abs();
            let ua: u8 = signedness(a);
            let ub: u8 = signedness(b);
            let unsigned_dist = (ua as i32 - ub as i32).unsigned_abs();
            prop_assert_eq!(
                signed_dist, unsigned_dist,
                "distance not preserved: {} - {} = {}, but {} - {} = {}",
                a, b, signed_dist, ua, ub, unsigned_dist
            );
        }

        /// Distance preservation for i32 (using i64 for intermediate calculations)
        #[test]
        fn prop_distance_preservation_i32(a in proptest::num::i32::ANY, b in proptest::num::i32::ANY) {
            let signed_dist = (a as i64 - b as i64).unsigned_abs();
            let ua: u32 = signedness(a);
            let ub: u32 = signedness(b);
            let unsigned_dist = (ua as i64 - ub as i64).unsigned_abs();
            prop_assert_eq!(
                signed_dist, unsigned_dist,
                "distance not preserved: {} - {} = {}, but {} - {} = {}",
                a, b, signed_dist, ua, ub, unsigned_dist
            );
        }

        /// Boundary mapping: MIN maps to 0, MAX maps to MAX
        #[test]
        fn prop_boundary_min_maps_to_zero(_unused in Just(())) {
            prop_assert_eq!(signedness(i8::MIN), u8::MIN);
            prop_assert_eq!(signedness(i16::MIN), u16::MIN);
            prop_assert_eq!(signedness(i32::MIN), u32::MIN);
            prop_assert_eq!(signedness(i64::MIN), u64::MIN);
        }

        #[test]
        fn prop_boundary_max_maps_to_max(_unused in Just(())) {
            prop_assert_eq!(signedness(i8::MAX), u8::MAX);
            prop_assert_eq!(signedness(i16::MAX), u16::MAX);
            prop_assert_eq!(signedness(i32::MAX), u32::MAX);
            prop_assert_eq!(signedness(i64::MAX), u64::MAX);
        }

        /// Zero maps to midpoint (2^(bits-1))
        #[test]
        fn prop_zero_maps_to_midpoint(_unused in Just(())) {
            prop_assert_eq!(signedness(0i8), 128u8);
            prop_assert_eq!(signedness(0i16), 32768u16);
            prop_assert_eq!(signedness(0i32), 2147483648u32);
            prop_assert_eq!(signedness(0i64), 9223372036854775808u64);
        }
    }

    // ========================
    // Original Unit Tests
    // ========================

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
    #[test]
    fn specific_values_i8() {
        assert_eq!(signedness(i8::MIN), 0u8);
        assert_eq!(signedness(-1i8), 127u8);
        assert_eq!(signedness(0i8), 128u8);
        assert_eq!(signedness(i8::MAX), 255u8);
    }
    #[test]
    fn roundtrip_i16() {
        for i in i16::MIN..=i16::MAX {
            let u: u16 = signedness(i);
            let back: i16 = signedness(u);
            assert_eq!(i, back, "roundtrip failed for i16 {i}");
        }
    }
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
    #[test]
    fn boundary_mapping() {
        assert_eq!(signedness(i8::MIN), u8::MIN);
        assert_eq!(signedness(i8::MAX), u8::MAX);
        assert_eq!(signedness(i16::MIN), u16::MIN);
        assert_eq!(signedness(i16::MAX), u16::MAX);
        assert_eq!(signedness(i32::MIN), u32::MIN);
        assert_eq!(signedness(i32::MAX), u32::MAX);
        assert_eq!(signedness(i64::MIN), u64::MIN);
        assert_eq!(signedness(i64::MAX), u64::MAX);
        assert_eq!(signedness(i128::MIN), u128::MIN);
        assert_eq!(signedness(i128::MAX), u128::MAX);
    }
}
