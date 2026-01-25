#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between N-bit unsigned integers and pairs of N/2-bit signed integers
where unsigned integers are mapped onto points on successive squares (Chebyshev
L∞ shells), starting at (0, 0), with deterministic angular ordering within each
shell - spiraling outward from the origin.

In other words: as we start at 0 and look at ascending unsigned integers, we
first see (0, 0), then all points on the shell with max(|x|, |y|) = 1 in
counterclockwise order starting from the positive x-axis, then shell 2, etc.
        "#
    };
}
// spell-checker: disable
use description;

impl_with!(u16, i8, u8, 8);
impl_with!(u32, i16, u16, 16);
impl_with!(u64, i32, u32, 32);

#[doc = description!()]
pub fn spiral_square<T: SpiralSquare>(value: T) -> T::Out
where
    T: Copy + PartialEq + core::fmt::Debug,
    T::Out: SpiralSquare<Out = T> + Copy,
{
    let result = value.spiral_square_impl();
    #[cfg(fuzzing)]
    {
        let roundtrip = result.spiral_square_impl();
        debug_assert_eq!(roundtrip, value, "spiral_square roundtrip failed");
    }
    result
}

#[doc = description!()]
pub trait SpiralSquare {
    type Out;
    /// Internal implementation - use `spiral_square()` function instead for fuzz-checked version
    #[doc(hidden)]
    fn spiral_square_impl(self) -> Self::Out;
}

macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr) => {
        impl SpiralSquare for $U {
            type Out = ($S, $S);

            fn spiral_square_impl(self) -> Self::Out {
                to_xy::<$U, $S, $UB, $W>(self)
            }
        }
        impl SpiralSquare for ($S, $S) {
            type Out = $U;

            fn spiral_square_impl(self) -> Self::Out {
                from_xy::<$U, $S, $UB, $W>(self.0, self.1)
            }
        }
    };
}
use impl_with;
/// Convert unsigned index to (x, y) signed coordinates.
fn to_xy<U, S, _UB, const W: u32>(u: U) -> (S, S)
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + TryFrom<i64>,
    _UB: Copy,
    <S as TryFrom<i64>>::Error: std::fmt::Debug,
{
    let u: u64 = u.into();
    let min_s = -(1i64 << (W - 1));
    let region_a_size = ((1u64 << W) - 1) * ((1u64 << W) - 1);
    if u < region_a_size {
        if u == 0 {
            return (S::try_from(0).unwrap(), S::try_from(0).unwrap());
        }
        let r = isqrt(u);
        let m = r.div_ceil(2);
        let side = 2 * m - 1;
        let base = side * side;
        let idx_in_shell = u - base;
        let (x, y) = perimeter_point(m as i64, idx_in_shell);
        (S::try_from(x).unwrap(), S::try_from(y).unwrap())
    } else {
        let r = u - region_a_size;
        let two_w = 1u64 << W;
        if r < two_w {
            let y_offset = r as i64;
            let y = min_s + y_offset;
            (S::try_from(min_s).unwrap(), S::try_from(y).unwrap())
        } else {
            let k = r - two_w;
            let x = min_s + 1 + k as i64;
            (S::try_from(x).unwrap(), S::try_from(min_s).unwrap())
        }
    }
}
/// Convert (x, y) signed coordinates to unsigned index.
fn from_xy<U, S, _UB, const W: u32>(x: S, y: S) -> U
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + Into<i64>,
    _UB: Copy,
    <U as TryFrom<u64>>::Error: std::fmt::Debug,
{
    let x: i64 = x.into();
    let y: i64 = y.into();
    let min_s = -(1i64 << (W - 1));
    let region_a_size = ((1u64 << W) - 1) * ((1u64 << W) - 1);
    if x == min_s || y == min_s {
        let two_w = 1u64 << W;
        if x == min_s {
            let r = (y - min_s) as u64;
            return U::try_from(region_a_size + r).unwrap();
        } else {
            let k = (x - min_s - 1) as u64;
            return U::try_from(region_a_size + two_w + k).unwrap();
        }
    }
    let ax = x.unsigned_abs();
    let ay = y.unsigned_abs();
    let m = ax.max(ay);
    if m == 0 {
        return U::try_from(0u64).unwrap();
    }
    let base = (2 * m - 1) * (2 * m - 1);
    let idx_in_shell = perimeter_index(m as i64, x, y);
    U::try_from(base + idx_in_shell).unwrap()
}
/// Map index j in [0, 8m) to point on shell with max(|x|, |y|) = m.
/// Counterclockwise from (m, 0).
fn perimeter_point(m: i64, j: u64) -> (i64, i64) {
    let s = 2 * m as u64;
    if j < s {
        (m, -(m) + 1 + j as i64)
    } else if j < 2 * s {
        let jj = j - s;
        (m - 1 - jj as i64, m)
    } else if j < 3 * s {
        let jj = j - 2 * s;
        (-m, m - 1 - jj as i64)
    } else {
        let jj = j - 3 * s;
        (-m + 1 + jj as i64, -m)
    }
}
/// Inverse of perimeter_point: given a point on shell m, return its index j.
fn perimeter_index(m: i64, x: i64, y: i64) -> u64 {
    if x == m && y > -m {
        (y + m - 1) as u64
    } else if y == m && x < m {
        (3 * m - 1 - x) as u64
    } else if x == -m && y < m {
        (5 * m - 1 - y) as u64
    } else {
        (x + 7 * m - 1) as u64
    }
}
/// Integer square root (floor).
fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_u16_to_pair() {
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            let back: u16 = spiral_square((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }
    #[test]
    fn roundtrip_pair_to_u16() {
        for x in i8::MIN..=i8::MAX {
            for y in i8::MIN..=i8::MAX {
                let u: u16 = spiral_square((x, y));
                let (back_x, back_y): (i8, i8) = spiral_square(u);
                assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({x}, {y})");
            }
        }
    }
    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(i8, i8)> = HashSet::new();
        for u in 0u16..=u16::MAX {
            let pair: (i8, i8) = spiral_square(u);
            assert!(seen.insert(pair), "duplicate output for u16 {u}: {pair:?}");
        }
        assert_eq!(seen.len(), 65536);
    }
    #[test]
    fn zero_maps_to_origin() {
        let (x, y): (i8, i8) = spiral_square(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }
    #[test]
    fn shell_structure() {
        let mut shell_counts: [u32; 129] = [0; 129];
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            let shell = (x as i32).abs().max((y as i32).abs()) as usize;
            shell_counts[shell] += 1;
        }
        assert_eq!(shell_counts[0], 1, "shell 0 should have 1 point");
        for m in 1..=127usize {
            let expected = 8 * m as u32;
            assert_eq!(
                shell_counts[m], expected,
                "shell {m} should have {expected} points, got {}",
                shell_counts[m]
            );
        }
        assert_eq!(shell_counts[128], 511, "shell 128 should have 511 points");
    }
    #[test]
    fn magnitude_monotonic() {
        let region_a_size = 65025u16;
        let mut max_shell = 0i32;
        for u in 0u16..region_a_size {
            let (x, y): (i8, i8) = spiral_square(u);
            let shell = (x as i32).abs().max((y as i32).abs());
            assert!(
                shell >= max_shell,
                "shell decreased at u={u}: was {max_shell}, now {shell}"
            );
            max_shell = shell;
        }
    }
    #[test]
    fn angular_order_within_shell() {
        let shell1: Vec<(i8, i8)> = (1u16..9).map(spiral_square).collect();
        let expected = vec![
            (1i8, 0i8),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        assert_eq!(
            shell1, expected,
            "shell 1 should be in counterclockwise order"
        );
    }
    #[test]
    fn region_b_points() {
        let region_a_size = 65025u16;
        let mut region_b_points: Vec<(i8, i8)> = Vec::new();
        for u in region_a_size..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            region_b_points.push((x, y));
        }
        assert_eq!(region_b_points.len(), 511);
        for (x, y) in &region_b_points {
            assert!(
                *x == i8::MIN || *y == i8::MIN,
                "region B point ({x}, {y}) doesn't involve MIN"
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
            let (x, y): (i16, i16) = spiral_square(u);
            let back: u32 = spiral_square((x, y));
            assert_eq!(u, back, "roundtrip failed for u32 {u}");
        }
    }
    #[test]
    fn roundtrip_u64_sample() {
        let test_values: Vec<u64> = (0..1000)
            .chain((u64::MAX - 1000)..=u64::MAX)
            .chain((0..10000).map(|i| i * 1844674407370955))
            .collect();
        for u in test_values {
            let (x, y): (i32, i32) = spiral_square(u);
            let back: u64 = spiral_square((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }
}
