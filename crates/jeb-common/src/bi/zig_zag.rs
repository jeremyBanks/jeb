#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between signed and unsigned integer types which preserves the
ordering by magnitude (with each negative value considered lower-magnitude than
its corresponding positive value). Called ZigZag by the Protocol Buffers wire
protocol.
        "#
    };
}
use description;

#[doc = description!()]
pub fn zig_zag<T: ZigZag>(value: T) -> T::Out {
    value.zig_zag()
}

impls! {
    i8: u8;
    i16: u16;
    i64: u64;
    i128: u128;
    isize: usize;
}

#[doc = description!()]
pub trait ZigZag {
    type Out;

    #[doc = description!()]
    fn zig_zag(self) -> Self::Out;
}

macro_rules! impls {
        {$( $signed:ident: $unsigned:ident; )+} => {$(
        impl ZigZag for $signed {
            type Out = $unsigned;

            fn zig_zag(self) -> $unsigned {
                let sign_mask = (self >> (<$signed>::BITS - 1)) as $unsigned;
                 ((self as $unsigned) << 1) ^ sign_mask
            }
        }

        impl ZigZag for $unsigned {
            type Out = $signed;

            fn zig_zag(self) -> $signed {
                let lsb: $signed = (self & 1) as $signed;
                let neg_mask: $signed = -lsb;
                ((self >> 1) as $signed) ^ neg_mask
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
            let u: u8 = zig_zag(i);
            let back: i8 = zig_zag(u);
            assert_eq!(i, back, "roundtrip failed for i8 {i}");
        }
    }

    #[test]
    fn roundtrip_u8_to_i8() {
        for u in u8::MIN..=u8::MAX {
            let i: i8 = zig_zag(u);
            let back: u8 = zig_zag(i);
            assert_eq!(u, back, "roundtrip failed for u8 {u}");
        }
    }

    // Test specific zig-zag encoding values
    // ZigZag encoding: 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, ...
    #[test]
    fn specific_values_i8() {
        assert_eq!(zig_zag(0i8), 0u8);
        assert_eq!(zig_zag(-1i8), 1u8);
        assert_eq!(zig_zag(1i8), 2u8);
        assert_eq!(zig_zag(-2i8), 3u8);
        assert_eq!(zig_zag(2i8), 4u8);
        assert_eq!(zig_zag(-3i8), 5u8);
        assert_eq!(zig_zag(3i8), 6u8);
        // Boundaries
        assert_eq!(zig_zag(i8::MAX), 254u8); // 127 -> 254
        assert_eq!(zig_zag(i8::MIN), 255u8); // -128 -> 255
    }

    #[test]
    fn specific_values_u8_to_i8() {
        assert_eq!(zig_zag(0u8), 0i8);
        assert_eq!(zig_zag(1u8), -1i8);
        assert_eq!(zig_zag(2u8), 1i8);
        assert_eq!(zig_zag(3u8), -2i8);
        assert_eq!(zig_zag(4u8), 2i8);
        assert_eq!(zig_zag(254u8), 127i8);
        assert_eq!(zig_zag(255u8), -128i8);
    }

    // Test magnitude ordering: smaller |n| maps to smaller unsigned
    // With ties broken by: negative comes before positive
    #[test]
    fn magnitude_ordering_i8() {
        // Check that for |a| < |b|, zig_zag(a) < zig_zag(b)
        // And for |a| == |b| where a < 0 and b > 0, zig_zag(a) < zig_zag(b)
        for n in 0i8..127i8 {
            let neg = -n - 1; // -1, -2, -3, ...
            let pos = n + 1;  // 1, 2, 3, ...
            let u_neg: u8 = zig_zag(neg);
            let u_pos: u8 = zig_zag(pos);
            // Same magnitude, negative comes first
            assert!(
                u_neg < u_pos,
                "magnitude ordering: {neg} should map before {pos}, got {u_neg} vs {u_pos}"
            );
        }
    }

    // Test bijection coverage
    #[test]
    fn bijection_coverage_i8() {
        use std::collections::HashSet;
        let mut seen: HashSet<u8> = HashSet::new();
        for i in i8::MIN..=i8::MAX {
            let u: u8 = zig_zag(i);
            assert!(seen.insert(u), "duplicate output for i8 {i}: {u}");
        }
        assert_eq!(seen.len(), 256);
    }

    // Test roundtrip for i16
    #[test]
    fn roundtrip_i16() {
        for i in i16::MIN..=i16::MAX {
            let u: u16 = zig_zag(i);
            let back: i16 = zig_zag(u);
            assert_eq!(i, back, "roundtrip failed for i16 {i}");
        }
    }

    // Test specific i16 values
    #[test]
    fn specific_values_i16() {
        assert_eq!(zig_zag(0i16), 0u16);
        assert_eq!(zig_zag(-1i16), 1u16);
        assert_eq!(zig_zag(1i16), 2u16);
        assert_eq!(zig_zag(i16::MAX), (u16::MAX - 1)); // 32767 -> 65534
        assert_eq!(zig_zag(i16::MIN), u16::MAX);       // -32768 -> 65535
    }

    // Test roundtrip for larger types (sampled)
    #[test]
    fn roundtrip_i64_sample() {
        let test_values: Vec<i64> = (i64::MIN..(i64::MIN + 1000))
            .chain((i64::MAX - 1000)..=i64::MAX)
            .chain(-1000..1000)
            .collect();

        for i in test_values {
            let u: u64 = zig_zag(i);
            let back: i64 = zig_zag(u);
            assert_eq!(i, back, "roundtrip failed for i64 {i}");
        }
    }

    #[test]
    fn roundtrip_i128_sample() {
        let test_values: Vec<i128> = [i128::MIN, i128::MIN + 1, -1, 0, 1, i128::MAX - 1, i128::MAX]
            .into_iter()
            .collect();

        for i in test_values {
            let u: u128 = zig_zag(i);
            let back: i128 = zig_zag(u);
            assert_eq!(i, back, "roundtrip failed for i128 {i}");
        }
    }

    // Test boundary mapping for all types
    #[test]
    fn boundary_mapping() {
        // For zig-zag: MAX -> MAX-1, MIN -> MAX
        assert_eq!(zig_zag(i8::MAX), u8::MAX - 1);
        assert_eq!(zig_zag(i8::MIN), u8::MAX);
        assert_eq!(zig_zag(i16::MAX), u16::MAX - 1);
        assert_eq!(zig_zag(i16::MIN), u16::MAX);
        assert_eq!(zig_zag(i64::MAX), u64::MAX - 1);
        assert_eq!(zig_zag(i64::MIN), u64::MAX);
        assert_eq!(zig_zag(i128::MAX), u128::MAX - 1);
        assert_eq!(zig_zag(i128::MIN), u128::MAX);
    }
}
