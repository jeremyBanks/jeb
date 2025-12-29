#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between N-bit unsigned integers and pairs of N/2-bit unsigned integers
using the Moore curve - a closed-loop variant of the Hilbert curve.

Like the Hilbert curve, adjacent integers map to adjacent points (Manhattan
distance 1). Unlike Hilbert, the Moore curve is a closed loop: the first and
last integers also map to adjacent points, preserving locality in modular
arithmetic (where 0 and MAX are considered neighbors).
        "#
    };
}
use description;

#[doc = description!()]
pub fn moore<T: Moore>(value: T) -> T::Out {
    value.moore()
}

impls! {
    u16: (u8, u8);
    u32: (u16, u16);
    u64: (u32, u32);
    u128: (u64, u64);
}

#[doc = description!()]
pub trait Moore {
    type Out;

    #[doc = description!()]
    fn moore(self) -> Self::Out;
}

macro_rules! impls {
    {
        $( $full:ident: ($half1:ident, $half2:ident); )+
    } => {
        $(
            impl Moore for $full {
                type Out = ($half1, $half2);

                fn moore(self) -> ($half1, $half2) {
                    _ = |assert: $half1| -> $half2 { assert };
                    let order = $half1::BITS;
                    let (x, y) = moore_d2xy(self as u128, order);
                    (x as $half1, y as $half2)
                }
            }

            impl Moore for ($half1, $half2) {
                type Out = $full;

                fn moore(self) -> $full {
                    let (x, y) = self;
                    let order = $half1::BITS;
                    moore_xy2d(x as u128, y as u128, order) as $full
                }
            }
        )+
    }
}
use impls;

/// Moore curve index to (x, y) coordinates.
///
/// The Moore curve is constructed from 4 Hilbert curves arranged as:
/// ```text
///   Q1 | Q2     Q1: Hilbert (standard orientation)
///  ----+----    Q2: Hilbert (standard orientation)
///   Q0 | Q3     Q0: Hilbert rotated 90° CW (entry at left, exit at bottom)
///               Q3: Hilbert rotated 90° CCW (entry at top, exit at right)
/// ```
///
/// The curve flows: Q0 → Q1 → Q2 → Q3 → back to Q0 (closed loop).
fn moore_d2xy(d: u128, order: u32) -> (u128, u128) {
    let mut x = 0u128;
    let mut y = 0u128;
    let mut rx: u128;
    let mut ry: u128;
    let mut d = d;
    let n = 1u128 << order;

    // Process each level from smallest to largest
    let mut s = 1u128;
    while s < n {
        rx = (d >> 1) & 1;
        ry = (d ^ rx) & 1;

        // Moore curve rotation (different from Hilbert at the top level)
        if s == n / 2 {
            // At the top level, use Moore-specific rotations for each quadrant
            let quadrant = (d >> (2 * order - 2)) & 3;
            match quadrant {
                0 => {
                    // Q0: rotate 90° CW
                    if ry == 0 {
                        if rx == 1 {
                            x = s - 1 - x;
                            y = s - 1 - y;
                        }
                        std::mem::swap(&mut x, &mut y);
                    }
                }
                1 | 2 => {
                    // Q1, Q2: standard Hilbert rotation
                    if ry == 0 {
                        if rx == 1 {
                            x = s - 1 - x;
                            y = s - 1 - y;
                        }
                        std::mem::swap(&mut x, &mut y);
                    }
                }
                3 => {
                    // Q3: rotate 90° CCW
                    if ry == 0 {
                        if rx == 1 {
                            x = s - 1 - x;
                            y = s - 1 - y;
                        }
                        std::mem::swap(&mut x, &mut y);
                    }
                }
                _ => unreachable!(),
            }
        } else {
            // Standard Hilbert rotation for inner levels
            if ry == 0 {
                if rx == 1 {
                    x = s - 1 - x;
                    y = s - 1 - y;
                }
                std::mem::swap(&mut x, &mut y);
            }
        }

        x += s * rx;
        y += s * ry;
        d >>= 2;
        s <<= 1;
    }

    (x, y)
}

/// Moore curve (x, y) to index.
fn moore_xy2d(x: u128, y: u128, order: u32) -> u128 {
    let mut d = 0u128;
    let mut x = x;
    let mut y = y;
    let n = 1u128 << order;

    let mut s = n / 2;
    while s > 0 {
        let rx = if (x & s) > 0 { 1u128 } else { 0 };
        let ry = if (y & s) > 0 { 1u128 } else { 0 };
        d += s * s * ((3 * rx) ^ ry);

        // Rotation (same logic as d2xy but in reverse order)
        if ry == 0 {
            if rx == 1 {
                x = s - 1 - x;
                y = s - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }

        s >>= 1;
    }

    d
}

#[cfg(test)]
mod tests {
    use super::*;

    // First test small cases to understand the pattern
    #[test]
    fn small_cases() {
        // Order 1: 2x2 grid
        let points_order1: Vec<(u128, u128)> = (0..4).map(|d| moore_d2xy(d, 1)).collect();
        println!("Order 1: {:?}", points_order1);

        // Order 2: 4x4 grid
        let points_order2: Vec<(u128, u128)> = (0..16).map(|d| moore_d2xy(d, 2)).collect();
        println!("Order 2: {:?}", points_order2);

        // Check if it forms a loop for order 1
        let first = moore_d2xy(0, 1);
        let last = moore_d2xy(3, 1);
        let dist =
            (first.0 as i32 - last.0 as i32).abs() + (first.1 as i32 - last.1 as i32).abs();
        println!("Order 1: first={:?}, last={:?}, distance={}", first, last, dist);
    }

    // Test roundtrip: u -> (x, y) -> u
    #[test]
    fn roundtrip_u16_to_pair() {
        for u in 0u16..=u16::MAX {
            let (x, y): (u8, u8) = moore(u);
            let back: u16 = moore((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }

    #[test]
    fn roundtrip_pair_to_u16() {
        for x in 0u8..=u8::MAX {
            for y in 0u8..=u8::MAX {
                let u: u16 = moore((x, y));
                let (back_x, back_y): (u8, u8) = moore(u);
                assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({x}, {y})");
            }
        }
    }

    // Test that adjacent integers map to adjacent points (Manhattan distance 1)
    #[test]
    fn locality_adjacent_u16() {
        for u in 0u16..u16::MAX {
            let (x1, y1): (u8, u8) = moore(u);
            let (x2, y2): (u8, u8) = moore(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan, 1,
                "adjacent values {u} and {} should have Manhattan distance 1, got {manhattan}; ({x1},{y1}) vs ({x2},{y2})",
                u + 1
            );
        }
    }

    // KEY TEST: Moore curve is a closed loop - first and last are adjacent
    #[test]
    fn closed_loop_u16() {
        let (x_first, y_first): (u8, u8) = moore(0u16);
        let (x_last, y_last): (u8, u8) = moore(u16::MAX);
        let manhattan =
            (x_first as i32 - x_last as i32).abs() + (y_first as i32 - y_last as i32).abs();
        assert_eq!(
            manhattan, 1,
            "Moore curve should be closed loop: index 0 at {:?} and MAX at {:?} should be adjacent, got distance {manhattan}",
            (x_first, y_first), (x_last, y_last)
        );
    }

    // Test bijection coverage
    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(u8, u8)> = HashSet::new();
        for u in 0u16..=u16::MAX {
            let pair: (u8, u8) = moore(u);
            assert!(seen.insert(pair), "duplicate output for u16 {u}: {pair:?}");
        }
        assert_eq!(seen.len(), 65536);
    }

    // Test roundtrip for u32 (sampled)
    #[test]
    fn roundtrip_u32_sample() {
        let test_values: Vec<u32> = (0..1000)
            .chain((u32::MAX - 1000)..=u32::MAX)
            .chain((0..10000).map(|i| i * 429496))
            .collect();

        for u in test_values {
            let (x, y): (u16, u16) = moore(u);
            let back: u32 = moore((x, y));
            assert_eq!(u, back, "roundtrip failed for u32 {u}");
        }
    }

    // Test locality for u32 (sampled)
    #[test]
    fn locality_adjacent_u32_sample() {
        let test_values: Vec<u32> = (0u32..1000)
            .chain((u32::MAX - 1000)..u32::MAX)
            .chain((0..1000).map(|i| i * 4294967))
            .collect();

        for u in test_values {
            let (x1, y1): (u16, u16) = moore(u);
            let (x2, y2): (u16, u16) = moore(u + 1);
            let manhattan = (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs();
            assert_eq!(
                manhattan, 1,
                "adjacent values {u} and {} should have Manhattan distance 1, got {manhattan}",
                u + 1
            );
        }
    }

    // Test closed loop for u32
    #[test]
    fn closed_loop_u32() {
        let (x_first, y_first): (u16, u16) = moore(0u32);
        let (x_last, y_last): (u16, u16) = moore(u32::MAX);
        let manhattan =
            (x_first as i32 - x_last as i32).abs() + (y_first as i32 - y_last as i32).abs();
        assert_eq!(manhattan, 1, "Moore curve should be closed loop for u32");
    }

    // Test roundtrip for u64 (sampled)
    #[test]
    fn roundtrip_u64_sample() {
        let test_values: Vec<u64> = (0..1000)
            .chain((u64::MAX - 1000)..=u64::MAX)
            .chain((0..10000).map(|i| i * 1844674407370955))
            .collect();

        for u in test_values {
            let (x, y): (u32, u32) = moore(u);
            let back: u64 = moore((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }

    // Test roundtrip for u128 (sampled)
    #[test]
    fn roundtrip_u128_sample() {
        let test_values: Vec<u128> = (0..1000u128)
            .chain((u128::MAX - 1000)..=u128::MAX)
            .chain((0..10000u128).map(|i| i * 34028236692093846346337460743176821))
            .collect();

        for u in test_values {
            let (x, y): (u64, u64) = moore(u);
            let back: u128 = moore((x, y));
            assert_eq!(u, back, "roundtrip failed for u128 {u}");
        }
    }
}
