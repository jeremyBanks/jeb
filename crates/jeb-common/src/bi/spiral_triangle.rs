#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between N-bit unsigned integers and pairs of N/2-bit signed integers
where x ≤ y, enumerated by L∞ shells (Chebyshev distance from origin) with
deterministic angular ordering within each half-shell - spiraling outward from
the origin along the x ≤ y half-plane.

In other words: as we start at 0 and look at ascending unsigned integers, we
first see (0, 0), then all points on the shell with max(|x|, |y|) = 1 where
x ≤ y in counterclockwise order starting from (1, 1), then shell 2, etc.

For shell m > 0, the half-shell (x ≤ y) contains 4m + 1 points:
- Corner: (m, m)
- Top edge going left: (m-1, m), (m-2, m), ..., (-m, m) — 2m points
- Left edge going down: (-m, m-1), (-m, m-2), ..., (-m, -m) — 2m points
        "#
    };
}
// spell-checker: disable
use description;
impl_with!(u16, i8, u8, 8);
impl_with!(u32, i16, u16, 16);
impl_with!(u64, i32, u32, 32);
#[doc = description!()]
pub fn spiral_triangle<T: SpiralTriangle>(value: T) -> T::Out {
    value.spiral_triangle()
}
#[doc = description!()]
pub trait SpiralTriangle {
    type Out;
    #[doc = description!()]
    fn spiral_triangle(self) -> Self::Out;
}
macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr) => {
        impl SpiralTriangle for $U {
            type Out = ($S, $S);

            fn spiral_triangle(self) -> Self::Out {
                to_xy::<$U, $S, $UB, $W>(self)
            }
        }
        impl SpiralTriangle for ($S, $S) {
            type Out = $U;

            fn spiral_triangle(self) -> Self::Out {
                from_xy::<$U, $S, $UB, $W>(self.0, self.1)
            }
        }
    };
}
use impl_with;

/// Shell size for half-shell m (x ≤ y constraint).
/// Shell 0 has 1 point, shell m > 0 has 4m + 1 points.
#[inline(always)]
#[allow(dead_code)] // Used in tests and for documentation
fn half_shell_size(m: u64) -> u64 {
    if m == 0 { 1 } else { 4 * m + 1 }
}

/// Base index for half-shell m.
/// This is the sum of all half-shell sizes for shells 0 to m-1.
/// For m > 0: base(m) = 1 + sum(4k+1 for k=1 to m-1) = m * (2m - 1)
#[inline(always)]
fn half_shell_base(m: u64) -> u64 {
    if m == 0 { 0 } else { m * (2 * m - 1) }
}

/// Given an index u, determine which half-shell it belongs to.
/// Returns (shell_number, index_within_shell).
#[inline(always)]
fn index_to_shell(u: u64) -> (u64, u64) {
    if u == 0 {
        return (0, 0);
    }
    // base(m) = m * (2m - 1) = 2m² - m
    // We need to find m such that base(m) ≤ u < base(m+1)
    // base(m+1) = (m+1) * (2(m+1) - 1) = (m+1)(2m+1) = 2m² + 3m + 1
    // So: 2m² - m ≤ u < 2m² + 3m + 1
    // Using the quadratic formula to find m from u ≈ 2m² - m:
    // m ≈ (1 + sqrt(1 + 8u)) / 4
    let approx = ((1.0 + (1.0 + 8.0 * u as f64).sqrt()) / 4.0).floor() as u64;
    // Check approx and neighbors due to floating point imprecision
    let m = if half_shell_base(approx + 1) <= u {
        approx + 1
    } else if half_shell_base(approx) > u && approx > 0 {
        approx - 1
    } else {
        approx
    };
    (m, u - half_shell_base(m))
}

/// Map index j in [0, 4m+1) to point on half-shell with max(|x|, |y|) = m where
/// x ≤ y. Order: (m, m), then top edge left to (-m, m), then left edge down to
/// (-m, -m).
#[inline(always)]
fn half_perimeter_point(m: i64, j: u64) -> (i64, i64) {
    if j == 0 {
        // Corner point
        (m, m)
    } else if j <= 2 * m as u64 {
        // Top edge: y = m, x from m-1 down to -m
        let x = m - j as i64;
        (x, m)
    } else {
        // Left edge: x = -m, y from m-1 down to -m
        let jj = j - 2 * m as u64;
        let y = m - jj as i64;
        (-m, y)
    }
}

/// Inverse of half_perimeter_point: given a point on half-shell m where x ≤ y,
/// return its index j in [0, 4m+1).
#[inline(always)]
fn half_perimeter_index(m: i64, x: i64, y: i64) -> u64 {
    debug_assert!(x <= y, "half_perimeter_index requires x <= y");
    debug_assert!(
        x.abs().max(y.abs()) == m,
        "point ({x}, {y}) not on shell {m}"
    );
    if x == m {
        // Must be corner (m, m)
        debug_assert_eq!(y, m);
        0
    } else if y == m {
        // Top edge
        (m - x) as u64
    } else {
        // Left edge (x == -m)
        debug_assert_eq!(x, -m);
        (2 * m + (m - y)) as u64
    }
}

/// Convert unsigned index to (x, y) signed coordinates where x ≤ y.
#[expect(clippy::extra_unused_type_parameters, reason = "macro-generated signature")]
fn to_xy<U, S, _UB, const W: u32>(u: U) -> (S, S)
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + TryFrom<i64>,
    _UB: Copy,
    <S as TryFrom<i64>>::Error: std::fmt::Debug,
{
    let u: u64 = u.into();
    let min_s = -(1i64 << (W - 1));
    let max_s = (1i64 << (W - 1)) - 1;

    // Region A: complete half-shells from 0 to max_s
    // Total points in region A = (max_s + 1) * (2 * max_s + 1) for the triangle
    // But we need to calculate this properly using half_shell_base
    let region_a_shells = max_s as u64 + 1; // shells 0 to max_s
    let region_a_size = half_shell_base(region_a_shells);

    if u < region_a_size {
        let (m, idx_in_shell) = index_to_shell(u);
        if m == 0 {
            return (S::try_from(0).unwrap(), S::try_from(0).unwrap());
        }
        let (x, y) = half_perimeter_point(m as i64, idx_in_shell);
        (S::try_from(x).unwrap(), S::try_from(y).unwrap())
    } else {
        // Region B: pairs involving MIN
        // These are (MIN, y) for all valid y from MIN to MAX (2^W values)
        let r = u - region_a_size;
        let two_w = 1u64 << W;
        debug_assert!(r < two_w, "index out of range for triangle bijection");
        // Map r to y value: r=0 -> MIN, r=2^W-1 -> MAX
        let y = min_s + r as i64;
        (S::try_from(min_s).unwrap(), S::try_from(y).unwrap())
    }
}

/// Convert (x, y) signed coordinates where x ≤ y to unsigned index.
#[expect(clippy::extra_unused_type_parameters, reason = "macro-generated signature")]
fn from_xy<U, S, _UB, const W: u32>(x: S, y: S) -> U
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + Into<i64>,
    _UB: Copy,
    <U as TryFrom<u64>>::Error: std::fmt::Debug,
{
    let x: i64 = x.into();
    let y: i64 = y.into();
    debug_assert!(x <= y, "spiral_triangle requires x <= y, got ({x}, {y})");

    let min_s = -(1i64 << (W - 1));
    let max_s = (1i64 << (W - 1)) - 1;
    let region_a_shells = max_s as u64 + 1;
    let region_a_size = half_shell_base(region_a_shells);

    if x == min_s {
        // Region B: (MIN, y)
        let r = (y - min_s) as u64;
        return U::try_from(region_a_size + r).unwrap();
    }

    // Region A: x > MIN
    let ax = x.unsigned_abs();
    let ay = y.unsigned_abs();
    let m = ax.max(ay);
    if m == 0 {
        return U::try_from(0u64).unwrap();
    }
    let base = half_shell_base(m);
    let idx_in_shell = half_perimeter_index(m as i64, x, y);
    U::try_from(base + idx_in_shell).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_u16_to_pair() {
        // Test all valid triangle indices
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = spiral_triangle(u);
            let back: u16 = spiral_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }

    #[test]
    fn roundtrip_pair_to_u16() {
        for x in i8::MIN..=i8::MAX {
            for y in x..=i8::MAX {
                let u: u16 = spiral_triangle((x, y));
                let (back_x, back_y): (i8, i8) = spiral_triangle(u);
                assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({x}, {y})");
            }
        }
    }

    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(i8, i8)> = HashSet::new();
        for u in 0u16..32896 {
            let pair: (i8, i8) = spiral_triangle(u);
            assert!(seen.insert(pair), "duplicate output for u16 {u}: {pair:?}");
        }
        assert_eq!(seen.len(), 32896);
    }

    #[test]
    fn x_le_y_invariant() {
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = spiral_triangle(u);
            assert!(x <= y, "invariant violated: {x} > {y} for index {u}");
        }
    }

    #[test]
    fn zero_maps_to_origin() {
        let (x, y): (i8, i8) = spiral_triangle(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }

    #[test]
    fn shell_structure() {
        let mut shell_counts: [u32; 129] = [0; 129];
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = spiral_triangle(u);
            let shell = (x as i32).abs().max((y as i32).abs()) as usize;
            shell_counts[shell] += 1;
        }
        assert_eq!(shell_counts[0], 1, "shell 0 should have 1 point");
        for m in 1..=127usize {
            let expected = 4 * m as u32 + 1;
            assert_eq!(
                shell_counts[m], expected,
                "shell {m} should have {expected} points, got {}",
                shell_counts[m]
            );
        }
        assert_eq!(
            shell_counts[128], 256,
            "shell 128 (region B) should have 256 points"
        );
    }

    #[test]
    fn magnitude_monotonic() {
        let region_a_size = 32640u16;
        let mut max_shell = 0i32;
        for u in 0u16..region_a_size {
            let (x, y): (i8, i8) = spiral_triangle(u);
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
        // Shell 1 should be: (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)
        let shell1: Vec<(i8, i8)> = (1u16..6).map(spiral_triangle).collect();
        let expected = vec![(1i8, 1i8), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
        assert_eq!(shell1, expected, "shell 1 should be in correct order");
    }

    #[test]
    fn region_b_points() {
        let region_a_size = 32640u16;
        let mut region_b_points: Vec<(i8, i8)> = Vec::new();
        for u in region_a_size..32896 {
            let (x, y): (i8, i8) = spiral_triangle(u);
            region_b_points.push((x, y));
        }
        assert_eq!(region_b_points.len(), 256);
        for (x, _y) in &region_b_points {
            assert_eq!(*x, i8::MIN, "region B point ({x}, {_y}) should have x=MIN");
        }
    }

    #[test]
    fn half_shell_formulas() {
        // Verify half-shell size formula
        assert_eq!(half_shell_size(0), 1);
        assert_eq!(half_shell_size(1), 5);
        assert_eq!(half_shell_size(2), 9);
        assert_eq!(half_shell_size(3), 13);

        // Verify half-shell base formula
        assert_eq!(half_shell_base(0), 0);
        assert_eq!(half_shell_base(1), 1);
        assert_eq!(half_shell_base(2), 6);
        assert_eq!(half_shell_base(3), 15);
        assert_eq!(half_shell_base(4), 28);

        // Verify consistency: base(m+1) = base(m) + size(m)
        for m in 0..100 {
            assert_eq!(
                half_shell_base(m + 1),
                half_shell_base(m) + half_shell_size(m),
                "base formula inconsistent at m={m}"
            );
        }
    }

    #[test]
    fn roundtrip_u32_sample() {
        let test_values: Vec<u32> = (0..10000).chain((0..10000).map(|i| i * 100000)).collect();
        for u in test_values {
            let (x, y): (i16, i16) = spiral_triangle(u);
            let back: u32 = spiral_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u32 {u}");
        }
    }

    #[test]
    fn roundtrip_u64_sample() {
        let test_values: Vec<u64> = (0..10000).chain((0..10000).map(|i| i * 100000)).collect();
        for u in test_values {
            let (x, y): (i32, i32) = spiral_triangle(u);
            let back: u64 = spiral_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }
}
