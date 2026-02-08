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
    use super::*;

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
}
