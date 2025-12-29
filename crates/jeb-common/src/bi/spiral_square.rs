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

This is similar to scatter_square but with predictable spiral ordering instead
of pseudorandom permutation within each shell.
        "#
    };
}
use description;

impl_with!(u16, i8, u8, 8);
impl_with!(u32, i16, u16, 16);
impl_with!(u64, i32, u32, 32);

#[doc = description!()]
pub fn spiral_square<T: SpiralSquare>(value: T) -> T::Out {
    value.spiral_square()
}

#[doc = description!()]
pub trait SpiralSquare {
    type Out;

    #[doc = description!()]
    fn spiral_square(self) -> Self::Out;
}

macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr) => {
        impl SpiralSquare for $U {
            type Out = ($S, $S);

            fn spiral_square(self) -> Self::Out {
                to_xy::<$U, $S, $UB, $W>(self)
            }
        }

        impl SpiralSquare for ($S, $S) {
            type Out = $U;

            fn spiral_square(self) -> Self::Out {
                from_xy::<$U, $S, $UB, $W>(self.0, self.1)
            }
        }
    };
}
use impl_with;

/// Convert unsigned index to (x, y) signed coordinates.
fn to_xy<U, S, UB, const W: u32>(u: U) -> (S, S)
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + TryFrom<i64>,
    UB: Copy,
    <S as TryFrom<i64>>::Error: std::fmt::Debug,
{
    let u: u64 = u.into();

    // Region A: the symmetric square [-MAX..MAX]^2
    // Region B: the ragged outer shell involving MIN

    let min_s = -(1i64 << (W - 1)); // e.g., -128 for i8

    // Region A size: (2*MAX+1)^2 = (2^W - 1)^2
    let region_a_size = ((1u64 << W) - 1) * ((1u64 << W) - 1);

    if u < region_a_size {
        // Region A: standard shells
        if u == 0 {
            return (S::try_from(0).unwrap(), S::try_from(0).unwrap());
        }

        // Find which shell: base(m) = (2m-1)^2, so m = floor((sqrt(u)+1)/2)
        let r = isqrt(u);
        let m = (r + 1) / 2; // shell number, 1..=MAX (floor division)

        // base(m) = (2m-1)^2 for m >= 1
        let side = 2 * m - 1;
        let base = side * side;
        let idx_in_shell = u - base;

        // Map idx_in_shell to a point on the perimeter
        // We go counterclockwise starting from (m, 0):
        // - Side 0: (m, 0) to (m, m-1) then to (m, m) [length 2m, from (m, -m+1) to (m, m)]
        //   Actually, let's define more carefully:
        //   Side 0 (right): x = m, y goes from 0 up to m, then -1 down to -m+1 [but that's weird]
        //
        // Standard perimeter traversal (counterclockwise from (m, 0)):
        //   - Right edge going up: (m, y) for y = 0, 1, ..., m-1, m
        //   - Top edge going left: (x, m) for x = m-1, m-2, ..., -m
        //   - Left edge going down: (-m, y) for y = m-1, m-2, ..., -m
        //   - Bottom edge going right: (x, -m) for x = -m+1, ..., m-1, m
        //   - Back to start at (m, 0)... wait, we started at (m, 0), not (m, -m)
        //
        // Let me redefine: start at (m, 1-m) and go counterclockwise:
        //   Actually, the simplest is to start at (m, -m) and go counterclockwise.
        //
        // Let's use: start at (m, -m+1) going up on right edge
        // - Right edge: (m, y) for y = -m+1, ..., m (length 2m)
        // - Top edge: (x, m) for x = m-1, ..., -m (length 2m)
        // - Left edge: (-m, y) for y = m-1, ..., -m (length 2m)
        // - Bottom edge: (x, -m) for x = -m+1, ..., m-1 (length 2m-1)
        //   Wait that's only 8m-1 points, not 8m.
        //
        // The perimeter of a square with max coord m has 8m points:
        //   4 sides, each with 2m points, but corners are shared
        //   Actually: (2m+1)^2 - (2m-1)^2 = 4m*2 = 8m ✓
        //
        // Simple approach: parameterize by angle-like position j ∈ [0, 8m)
        let (x, y) = perimeter_point(m as i64, idx_in_shell);
        (S::try_from(x).unwrap(), S::try_from(y).unwrap())
    } else {
        // Region B: ragged outer shell involving MIN
        let r = u - region_a_size;
        // Region B has 2^(W+1) - 1 points: all (MIN, y) plus (x, MIN) for x != MIN
        let two_w = 1u64 << W;

        if r < two_w {
            // (MIN, y) for all y
            let y_offset = r as i64;
            // y ranges over all values: MIN, MIN+1, ..., MAX
            // We want r=0 -> y=MIN, r=1 -> y=MIN+1, etc.
            let y = min_s + y_offset;
            (S::try_from(min_s).unwrap(), S::try_from(y).unwrap())
        } else {
            // (x, MIN) for x != MIN
            let k = r - two_w; // 0..2^W-2
            // x ranges over all values except MIN: MIN+1, ..., MAX
            let x = min_s + 1 + k as i64;
            (S::try_from(x).unwrap(), S::try_from(min_s).unwrap())
        }
    }
}

/// Convert (x, y) signed coordinates to unsigned index.
fn from_xy<U, S, UB, const W: u32>(x: S, y: S) -> U
where
    U: Copy + Into<u64> + TryFrom<u64>,
    S: Copy + Into<i64>,
    UB: Copy,
    <U as TryFrom<u64>>::Error: std::fmt::Debug,
{
    let x: i64 = x.into();
    let y: i64 = y.into();

    let min_s = -(1i64 << (W - 1));
    let region_a_size = ((1u64 << W) - 1) * ((1u64 << W) - 1);

    // Check for Region B (involves MIN)
    if x == min_s || y == min_s {
        let two_w = 1u64 << W;

        if x == min_s {
            // (MIN, y) -> r = y - MIN
            let r = (y - min_s) as u64;
            return U::try_from(region_a_size + r).unwrap();
        } else {
            // (x, MIN) where x != MIN
            let k = (x - min_s - 1) as u64;
            return U::try_from(region_a_size + two_w + k).unwrap();
        }
    }

    // Region A
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
    let s = 2 * m as u64; // side length in terms of steps

    if j < s {
        // Right edge going up: (m, -m + 1 + j) for j in [0, 2m)
        // j=0 -> (m, -m+1), j=2m-1 -> (m, m)
        // Actually let's start at (m, 0):
        // First quadrant: (m, 0), (m, 1), ..., (m, m-1), then corner (m, m)
        // That's m+1 points on the first part of right edge.
        // Wait, let me think again.
        //
        // The 8m points on shell m form the perimeter.
        // Let's use a clean parameterization:
        //
        // j in [0, 2m): right edge, x=m, y goes from -m+1 to m
        //   j=0: (m, -m+1), j=2m-1: (m, m)
        // j in [2m, 4m): top edge, y=m, x goes from m-1 to -m
        //   j=2m: (m-1, m), j=4m-1: (-m, m)
        // j in [4m, 6m): left edge, x=-m, y goes from m-1 to -m
        //   j=4m: (-m, m-1), j=6m-1: (-m, -m)
        // j in [6m, 8m): bottom edge, y=-m, x goes from -m+1 to m
        //   j=6m: (-m+1, -m), j=8m-1: (m, -m)
        //
        // But wait, (m, -m) should connect to (m, -m+1) which is j=0.
        // So j=8m-1 is (m, -m) and j=0 is (m, -m+1).
        // Distance between them: |(m, -m) - (m, -m+1)| = 1 ✓

        // Right edge: x = m, y = -m + 1 + j
        (m, -(m) + 1 + j as i64)
    } else if j < 2 * s {
        // Top edge: y = m, x = m - 1 - (j - 2m) = 2m - 1 - j
        let jj = j - s;
        (m - 1 - jj as i64, m)
    } else if j < 3 * s {
        // Left edge: x = -m, y = m - 1 - (j - 4m)
        let jj = j - 2 * s;
        (-m, m - 1 - jj as i64)
    } else {
        // Bottom edge: y = -m, x = -m + 1 + (j - 6m)
        let jj = j - 3 * s;
        (-m + 1 + jj as i64, -m)
    }
}

/// Inverse of perimeter_point: given a point on shell m, return its index j.
fn perimeter_index(m: i64, x: i64, y: i64) -> u64 {
    if x == m && y > -m {
        // Right edge: y = -m + 1 + j, so j = y - (-m + 1) = y + m - 1
        (y + m - 1) as u64
    } else if y == m && x < m {
        // Top edge: x = m - 1 - (j - 2m), so j = m - 1 - x + 2m = 3m - 1 - x
        // Wait, let me recalculate:
        // j in [2m, 4m), x = m - 1 - (j - 2m)
        // j - 2m = m - 1 - x
        // j = 3m - 1 - x
        (3 * m - 1 - x) as u64
    } else if x == -m && y < m {
        // Left edge: y = m - 1 - (j - 4m), so j = m - 1 - y + 4m = 5m - 1 - y
        (5 * m - 1 - y) as u64
    } else {
        // Bottom edge (y == -m, x > -m): x = -m + 1 + (j - 6m), so j = x + m - 1 + 6m = x + 7m - 1
        (x + 7 * m - 1) as u64
    }
}

/// Integer square root (floor).
fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test roundtrip: u16 -> (i8, i8) -> u16
    #[test]
    fn roundtrip_u16_to_pair() {
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            let back: u16 = spiral_square((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }

    // Test roundtrip: (i8, i8) -> u16 -> (i8, i8)
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

    // Test bijection coverage
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

    // Test that 0 maps to origin
    #[test]
    fn zero_maps_to_origin() {
        let (x, y): (i8, i8) = spiral_square(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }

    // Test shell structure: points in each shell
    #[test]
    fn shell_structure() {
        let mut shell_counts: [u32; 129] = [0; 129];
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            let shell = (x as i32).abs().max((y as i32).abs()) as usize;
            shell_counts[shell] += 1;
        }

        // Shell 0: 1 point (0,0)
        assert_eq!(shell_counts[0], 1, "shell 0 should have 1 point");

        // Shell M (1 <= M <= 127): 8*M points
        for m in 1..=127usize {
            let expected = 8 * m as u32;
            assert_eq!(
                shell_counts[m], expected,
                "shell {m} should have {expected} points, got {}",
                shell_counts[m]
            );
        }

        // Shell 128 (ragged): 511 points
        assert_eq!(shell_counts[128], 511, "shell 128 should have 511 points");
    }

    // Test that shells are filled in order (monotonically non-decreasing magnitude)
    #[test]
    fn magnitude_monotonic() {
        let region_a_size = 65025u16; // (255)^2

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

    // Test that within each shell, points are visited in angular order
    #[test]
    fn angular_order_within_shell() {
        // Check shell 1: should visit in counterclockwise order from (1, 0)
        // Shell 1 base = 1, size = 8
        // Expected order: (1,0), (1,1), (0,1), (-1,1), (-1,0), (-1,-1), (0,-1), (1,-1)
        // Wait, based on my perimeter_point:
        // j=0: (1, -1+1) = (1, 0) ✓
        // j=1: (1, 1) ✓
        // j=2: (0, 1) ✓
        // j=3: (-1, 1) ✓
        // j=4: (-1, 0) ✓
        // j=5: (-1, -1) ✓
        // j=6: (0, -1) ✓
        // j=7: (1, -1) ✓

        let shell1: Vec<(i8, i8)> = (1u16..9).map(|u| spiral_square(u)).collect();
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
        assert_eq!(shell1, expected, "shell 1 should be in counterclockwise order");
    }

    // Test region B (ragged outer shell)
    #[test]
    fn region_b_points() {
        let region_a_size = 65025u16;

        let mut region_b_points: Vec<(i8, i8)> = Vec::new();
        for u in region_a_size..=u16::MAX {
            let (x, y): (i8, i8) = spiral_square(u);
            region_b_points.push((x, y));
        }

        // Region B should have 511 points
        assert_eq!(region_b_points.len(), 511);

        // All region B points should involve i8::MIN (-128)
        for (x, y) in &region_b_points {
            assert!(
                *x == i8::MIN || *y == i8::MIN,
                "region B point ({x}, {y}) doesn't involve MIN"
            );
        }
    }

    // Test roundtrip for u32 (sampled)
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

    // Test roundtrip for u64 (sampled)
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
