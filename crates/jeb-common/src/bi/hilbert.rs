#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between between N-bit unsigned integers and pairs of N/2-bit unsigned
integers which preserves locality through an order-N/2 Hilbert curve: two
unsigned integers with a distance of X will encode into two pairs of unsigned
integers whose Manhattan distance is at less-than or equal to X and 3√X.
        "#
    };
}
// spell-checker: disable
use description;
#[doc = description!()]
pub fn hilbert<T: Hilbert>(value: T) -> T::Out {
    value.hilbert()
}
impls! {
     u16: ( u8,  u8);
     u32: (u16, u16);
     u64: (u32, u32);
    u128: (u64, u64);
}
#[doc = description!()]
pub trait Hilbert {
    type Out;
    #[doc = description!()]
    fn hilbert(self) -> Self::Out;
}
macro_rules! impls {
    {$($full:ident : ($half1:ident, $half2:ident);)+} => {
        $(
            impl Hilbert for $full {
                type Out = ($half1, $half2);
                fn hilbert(self) -> ($half1, $half2) {
                    _ = | assert : $half1 | -> $half2 { assert };
                    ::fast_hilbert::h2xy(self, $half1 ::BITS.try_into().unwrap())
                }
            }
            impl Hilbert for ($half1, $half2) {
                type Out = $full;
                fn hilbert(self) -> $full {
                    let (x, y) = self;
                    ::fast_hilbert::xy2h(x, y, $half1 ::BITS.try_into().unwrap())
                }
            }
        )+
    };
}
use impls;
#[cfg(test)]
mod tests {
    use {
        super::*,
        proptest::prelude::*,
    };

    #[test]
    fn roundtrip_u16_to_pair() {
        for u in 0u16..=u16::MAX {
            let (x, y): (u8, u8) = hilbert(u);
            let back: u16 = hilbert((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u}");
        }
    }
    #[test]
    fn roundtrip_pair_to_u16() {
        for x in 0u8..=u8::MAX {
            for y in 0u8..=u8::MAX {
                let u: u16 = hilbert((x, y));
                let (back_x, back_y): (u8, u8) = hilbert(u);
                assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({x}, {y})");
            }
        }
    }
    #[test]
    fn locality_adjacent_u16() {
        for u in 0u16..u16::MAX {
            let (x1, y1): (u8, u8) = hilbert(u);
            let (x2, y2): (u8, u8) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan,
                1,
                "adjacent values {u} and {} should have Manhattan distance 1, got {manhattan}",
                u + 1
            );
        }
    }
    #[test]
    fn roundtrip_u32_sample() {
        let test_values: Vec<u32> = (0..1000)
            .chain((u32::MAX - 1000)..=u32::MAX)
            .chain((0..10000).map(|i| i * 429496))
            .collect();
        for u in test_values {
            let (x, y): (u16, u16) = hilbert(u);
            let back: u32 = hilbert((x, y));
            assert_eq!(u, back, "roundtrip failed for u32 {u}");
        }
    }
    #[test]
    fn locality_adjacent_u32_sample() {
        let test_values: Vec<u32> = (0u32..1000)
            .chain((u32::MAX - 1000)..u32::MAX)
            .chain((0..1000).map(|i| i * 4294967))
            .collect();
        for u in test_values {
            let (x1, y1): (u16, u16) = hilbert(u);
            let (x2, y2): (u16, u16) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan,
                1,
                "adjacent values {u} and {} should have Manhattan distance 1, got {manhattan}",
                u + 1
            );
        }
    }
    #[test]
    fn roundtrip_u64_sample() {
        let test_values: Vec<u64> = (0..1000)
            .chain((u64::MAX - 1000)..=u64::MAX)
            .chain((0..10000).map(|i| i * 1844674407370955))
            .collect();
        for u in test_values {
            let (x, y): (u32, u32) = hilbert(u);
            let back: u64 = hilbert((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }
    #[test]
    fn roundtrip_u128_sample() {
        let test_values: Vec<u128> = (0..1000u128)
            .chain((u128::MAX - 1000)..=u128::MAX)
            .chain((0..10000u128).map(|i| i * 34028236692093846346337460743176821))
            .collect();
        for u in test_values {
            let (x, y): (u64, u64) = hilbert(u);
            let back: u128 = hilbert((x, y));
            assert_eq!(u, back, "roundtrip failed for u128 {u}");
        }
    }
    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(u8, u8)> = HashSet::new();
        for u in 0u16..=u16::MAX {
            let pair: (u8, u8) = hilbert(u);
            assert!(seen.insert(pair), "duplicate output for u16: {pair:?}");
        }
        assert_eq!(seen.len(), 65536);
    }

    // ========================
    // Property-Based Tests
    // ========================
    //
    // These tests use proptest to verify properties hold for randomly generated
    // values. Properties tested:
    // 1. Roundtrip: hilbert(hilbert(x)) == x
    // 2. Bijection: Every unique input maps to a unique output
    // 3. Locality: Adjacent values on the curve have Manhattan distance 1

    proptest! {
        /// Roundtrip property: encoding and decoding returns the original u16 value
        #[test]
        fn prop_roundtrip_u16(u in proptest::num::u16::ANY) {
            let (x, y): (u8, u8) = hilbert(u);
            let back: u16 = hilbert((x, y));
            prop_assert_eq!(u, back, "roundtrip failed for u16 {}", u);
        }

        /// Roundtrip property: decoding and encoding returns the original pair
        #[test]
        fn prop_roundtrip_pair_u8(x in proptest::num::u8::ANY, y in proptest::num::u8::ANY) {
            let u: u16 = hilbert((x, y));
            let (back_x, back_y): (u8, u8) = hilbert(u);
            prop_assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({}, {})", x, y);
        }

        /// Roundtrip property for u32
        #[test]
        fn prop_roundtrip_u32(u in proptest::num::u32::ANY) {
            let (x, y): (u16, u16) = hilbert(u);
            let back: u32 = hilbert((x, y));
            prop_assert_eq!(u, back, "roundtrip failed for u32 {}", u);
        }

        /// Roundtrip property for u64
        #[test]
        fn prop_roundtrip_u64(u in proptest::num::u64::ANY) {
            let (x, y): (u32, u32) = hilbert(u);
            let back: u64 = hilbert((x, y));
            prop_assert_eq!(u, back, "roundtrip failed for u64 {}", u);
        }

        /// Roundtrip property for u128
        #[test]
        fn prop_roundtrip_u128(u in proptest::num::u128::ANY) {
            let (x, y): (u64, u64) = hilbert(u);
            let back: u128 = hilbert((x, y));
            prop_assert_eq!(u, back, "roundtrip failed for u128 {}", u);
        }

        /// Locality property: Adjacent values have Manhattan distance 1
        /// This is a key property of Hilbert curves
        #[test]
        fn prop_locality_u16(u in 0u16..u16::MAX) {
            let (x1, y1): (u8, u8) = hilbert(u);
            let (x2, y2): (u8, u8) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            prop_assert_eq!(
                manhattan, 1,
                "adjacent values {} and {} should have Manhattan distance 1, got {}",
                u, u + 1, manhattan
            );
        }

        /// Locality property for u32
        #[test]
        fn prop_locality_u32(u in 0u32..u32::MAX) {
            let (x1, y1): (u16, u16) = hilbert(u);
            let (x2, y2): (u16, u16) = hilbert(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            prop_assert_eq!(
                manhattan, 1,
                "adjacent values {} and {} should have Manhattan distance 1, got {}",
                u, u + 1, manhattan
            );
        }

        /// First and last values of the curve
        #[test]
        fn prop_curve_endpoints_u16(_unused in Just(())) {
            let (x0, y0): (u8, u8) = hilbert(0u16);
            let (x_max, y_max): (u8, u8) = hilbert(u16::MAX);
            // First point should be (0, 0)
            prop_assert_eq!((x0, y0), (0, 0), "curve should start at (0, 0)");
            // Last point should be (255, 0) for a 256x256 grid
            prop_assert_eq!((x_max, y_max), (255, 0), "curve should end at (255, 0)");
        }

        /// Bijection: two different inputs should produce different outputs
        #[test]
        fn prop_bijection_u16(a in proptest::num::u16::ANY, b in proptest::num::u16::ANY) {
            prop_assume!(a != b);
            let pair_a: (u8, u8) = hilbert(a);
            let pair_b: (u8, u8) = hilbert(b);
            prop_assert_ne!(pair_a, pair_b, "different inputs {} and {} should produce different outputs", a, b);
        }

        /// Distance bounds: values that are close on the curve should be somewhat close in 2D
        /// The Hilbert curve guarantees |x1-x2| + |y1-y2| <= 3*sqrt(|h1-h2|) approximately
        #[test]
        fn prop_distance_bound_u16(a in proptest::num::u16::ANY, b in proptest::num::u16::ANY) {
            let (x1, y1): (u8, u8) = hilbert(a);
            let (x2, y2): (u8, u8) = hilbert(b);
            let curve_dist = a.abs_diff(b) as f64;
            let manhattan = ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as f64;
            // The Hilbert curve property: manhattan distance <= 3 * sqrt(curve_dist)
            let bound = 3.0 * curve_dist.sqrt();
            prop_assert!(
                manhattan <= bound + 1.0, // +1 for rounding tolerance
                "manhattan distance {} should be <= 3*sqrt({}) = {} for values {} and {}",
                manhattan, curve_dist, bound, a, b
            );
        }
    }
}
