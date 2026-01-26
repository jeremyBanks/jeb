#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between N-bit unsigned integers and pairs of N/2-bit signed integers
where x ≤ y, enumerated by L∞ shells (Chebyshev distance from origin) with
pseudorandom ordering within each half-shell.

In other words: as we start at 0 and look at ascending unsigned integers, we'll
first see all of the points whose maximum component magnitude is 0 and x ≤ y,
then all such points with max magnitude 1, then 2, etc, so we're filling up
concentric half-shells centered at the origin, one shell at a time, but picking
points within each shell using a weakly-pseudorandom ordering.

For shell m > 0, the half-shell (x ≤ y) contains 4m + 1 points:
- Corner: (m, m)
- Top edge going left: (m-1, m), (m-2, m), ..., (-m, m) — 2m points
- Left edge going down: (-m, m-1), (-m, m-2), ..., (-m, -m) — 2m points
        "#
    };
}
// spell-checker: disable
use description;
impl_with!(u16, i8, u8, 8, scatter_triangle_u16);
impl_with!(u32, i16, u16, 16, scatter_triangle_u32);
impl_with!(u64, i32, u32, 32, scatter_triangle_u64);
#[doc = description!()]
pub fn scatter_triangle<T: ScatterTriangle>(value: T) -> T::Out {
    value.scatter_triangle()
}
#[doc = description!()]
pub trait ScatterTriangle {
    type Out;
    #[doc = description!()]
    fn scatter_triangle(self) -> Self::Out;
}

// Feistel network constants (same as scatter_square)
const ROUNDS: u32 = 4;
const MIX_MUL1: u64 = 0xBF58_476D_1CE4_E5B9;
const MIX_MUL2: u64 = 0x94D0_49BB_1331_11EB;
const KEY_CONST: u64 = 0xD6E8_FEB8_6659_FD93;
const ROUND_CONST: u64 = 0x9E37_79B9_7F4A_7C15;

#[inline(always)]
fn mix64(mut z: u64) -> u64 {
    z ^= z >> 30;
    z = z.wrapping_mul(MIX_MUL1);
    z ^= z >> 27;
    z = z.wrapping_mul(MIX_MUL2);
    z ^= z >> 31;
    z
}

#[inline(always)]
fn key_for_layer(layer_id: u64, seed: u64) -> u64 {
    mix64(seed ^ layer_id.wrapping_mul(KEY_CONST))
}

#[inline(always)]
fn next_pow2(mut n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n + 1
}

#[inline(always)]
fn domain_bits_for(n: u64) -> u32 {
    let d = next_pow2(n);
    let mut bits = d.trailing_zeros();
    if bits % 2 == 1 {
        bits += 1;
    }
    bits
}

#[inline(always)]
fn feistel_pow2(x: u64, bits: u32, rounds: u32, key: u64) -> u64 {
    let half = bits / 2;
    let mask = (1u64 << half) - 1;
    let mut l = (x >> half) & mask;
    let mut r = x & mask;
    for round in 0..rounds {
        let f = mix64(r ^ key ^ ROUND_CONST.wrapping_mul(round as u64)) & mask;
        let new_l = r;
        let new_r = (l ^ f) & mask;
        l = new_l;
        r = new_r;
    }
    (l << half) | r
}

#[inline(always)]
fn feistel_pow2_inv(x: u64, bits: u32, rounds: u32, key: u64) -> u64 {
    let half = bits / 2;
    let mask = (1u64 << half) - 1;
    let mut l = (x >> half) & mask;
    let mut r = x & mask;
    for round in (0..rounds).rev() {
        let f = mix64(l ^ key ^ ROUND_CONST.wrapping_mul(round as u64)) & mask;
        let r_old = l;
        let l_old = (r ^ f) & mask;
        l = l_old;
        r = r_old;
    }
    (l << half) | r
}

#[inline(always)]
fn permute_layer(idx: u64, n: u64, layer_id: u64, seed: u64) -> u64 {
    let key = key_for_layer(layer_id, seed);
    let bits = domain_bits_for(n);
    let mut y = idx;
    loop {
        y = feistel_pow2(y, bits, ROUNDS, key);
        if y < n {
            return y;
        }
    }
}

#[inline(always)]
fn permute_layer_inv(idx: u64, n: u64, layer_id: u64, seed: u64) -> u64 {
    let key = key_for_layer(layer_id, seed);
    let bits = domain_bits_for(n);
    let mut y = idx;
    loop {
        y = feistel_pow2_inv(y, bits, ROUNDS, key);
        if y < n {
            return y;
        }
    }
}

macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr, $mod:ident) => {
        mod $mod {
            use super::*;
            const W: u32 = $W;
            const MAX_S: i64 = <$S>::MAX as i64;
            const MIN_S: i64 = <$S>::MIN as i64;

            /// Shell size for half-shell m (x ≤ y constraint).
            /// Shell 0 has 1 point, shell m > 0 has 4m + 1 points.
            #[inline(always)]
            fn half_shell_size(m: u64) -> u64 {
                if m == 0 { 1 } else { 4 * m + 1 }
            }

            /// Base index for half-shell m.
            /// For m > 0: base(m) = m * (2m - 1)
            #[inline(always)]
            fn half_shell_base(m: u64) -> u64 {
                if m == 0 { 0 } else { m * (2 * m - 1) }
            }

            /// Given an index u, determine which half-shell it belongs to.
            #[inline(always)]
            fn index_to_shell(u: u64) -> (u64, u64) {
                if u == 0 {
                    return (0, 0);
                }
                // base(m) = 2m² - m, solve for m
                let approx = ((1.0 + (1.0 + 8.0 * u as f64).sqrt()) / 4.0).floor() as u64;
                let m = if half_shell_base(approx + 1) <= u {
                    approx + 1
                } else if half_shell_base(approx) > u && approx > 0 {
                    approx - 1
                } else {
                    approx
                };
                (m, u - half_shell_base(m))
            }

            /// Map index j in [0, 4m+1) to point on half-shell m where x ≤ y.
            #[inline(always)]
            fn half_perimeter_point(m: i64, j: u64) -> (i64, i64) {
                if j == 0 {
                    (m, m)
                } else if j <= 2 * m as u64 {
                    let x = m - j as i64;
                    (x, m)
                } else {
                    let jj = j - 2 * m as u64;
                    let y = m - jj as i64;
                    (-m, y)
                }
            }

            /// Inverse of half_perimeter_point.
            #[inline(always)]
            fn half_perimeter_index(m: i64, x: i64, y: i64) -> u64 {
                debug_assert!(x <= y);
                if x == m {
                    0
                } else if y == m {
                    (m - x) as u64
                } else {
                    (2 * m + (m - y)) as u64
                }
            }

            /// Region A size: total number of half-shell points from shell 0 to MAX_S
            #[inline(always)]
            fn region_a_size() -> u64 {
                let region_a_shells = MAX_S as u64 + 1;
                half_shell_base(region_a_shells)
            }

            /// Outer layer id for region B
            #[inline(always)]
            fn outer_layer_id() -> u64 {
                (MAX_S as u64) + 1
            }

            /// Region B size: 2^W (all pairs (MIN, y))
            #[inline(always)]
            fn region_b_size() -> u64 {
                1u64 << W
            }

            /// u -> (x, y) where x ≤ y
            pub fn to_xy(u_in: $U) -> ($S, $S) {
                let u = u_in as u64;
                let n0 = region_a_size();

                if u < n0 {
                    // Region A: complete half-shells
                    let (m, idx_in_shell) = index_to_shell(u);
                    if m == 0 {
                        return (0 as $S, 0 as $S);
                    }
                    let n = half_shell_size(m);
                    let seed = 0;
                    let j = permute_layer(idx_in_shell, n, m, seed);
                    let (x, y) = half_perimeter_point(m as i64, j);
                    return (x as $S, y as $S);
                }

                // Region B: pairs (MIN, y)
                let r = u - n0;
                let n = region_b_size();
                debug_assert!(r < n);
                let layer_id = outer_layer_id();
                let seed = 0;
                let rp = permute_layer(r, n, layer_id, seed);
                // Map rp to y: rp=0 -> MIN, rp=2^W-1 -> MAX
                let y = MIN_S + rp as i64;
                (MIN_S as $S, y as $S)
            }

            /// (x, y) -> u where x ≤ y
            pub fn from_xy(x: $S, y: $S) -> $U {
                debug_assert!((x as i64) <= (y as i64), "scatter_triangle requires x <= y");

                let n0 = region_a_size();

                if (x as i64) == MIN_S {
                    // Region B
                    let n = region_b_size();
                    let layer_id = outer_layer_id();
                    let rp = ((y as i64) - MIN_S) as u64;
                    let seed = 0;
                    let r = permute_layer_inv(rp, n, layer_id, seed);
                    return (n0 + r) as $U;
                }

                // Region A
                let xi = x as i64;
                let yi = y as i64;
                let ax = xi.unsigned_abs();
                let ay = yi.unsigned_abs();
                let m = ax.max(ay);
                if m == 0 {
                    return 0 as $U;
                }
                let j = half_perimeter_index(m as i64, xi, yi);
                let n = half_shell_size(m);
                let seed = 0;
                let t = permute_layer_inv(j, n, m, seed);
                let u = half_shell_base(m) + t;
                u as $U
            }
        }
        impl ScatterTriangle for $U {
            type Out = ($S, $S);

            #[inline(always)]
            fn scatter_triangle(self) -> Self::Out {
                $mod::to_xy(self)
            }
        }
        impl ScatterTriangle for ($S, $S) {
            type Out = $U;

            #[inline(always)]
            fn scatter_triangle(self) -> Self::Out {
                $mod::from_xy(self.0, self.1)
            }
        }
    };
}
use impl_with;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_u16_to_pair() {
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = scatter_triangle(u);
            let back: u16 = scatter_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }

    #[test]
    fn roundtrip_pair_to_u16() {
        for x in i8::MIN..=i8::MAX {
            for y in x..=i8::MAX {
                let u: u16 = scatter_triangle((x, y));
                let (back_x, back_y): (i8, i8) = scatter_triangle(u);
                assert_eq!(
                    (x, y),
                    (back_x, back_y),
                    "roundtrip failed for ({x}, {y}) -> {u}"
                );
            }
        }
    }

    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(i8, i8)> = HashSet::new();
        for u in 0u16..32896 {
            let pair: (i8, i8) = scatter_triangle(u);
            assert!(seen.insert(pair), "duplicate output for u16 {u}: {pair:?}");
        }
        assert_eq!(seen.len(), 32896);
    }

    #[test]
    fn x_le_y_invariant() {
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = scatter_triangle(u);
            assert!(x <= y, "invariant violated: {x} > {y} for index {u}");
        }
    }

    #[test]
    fn zero_maps_to_origin() {
        let (x, y): (i8, i8) = scatter_triangle(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }

    #[test]
    fn shell_structure() {
        let mut shell_counts: [u32; 129] = [0; 129];
        for u in 0u16..32896 {
            let (x, y): (i8, i8) = scatter_triangle(u);
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
    fn shells_filled_in_order() {
        let region_a_size = 32640u16;
        let mut max_shell_seen = 0i32;
        for u in 0u16..region_a_size {
            let (x, y): (i8, i8) = scatter_triangle(u);
            let shell = (x as i32).abs().max((y as i32).abs());
            assert!(
                shell >= max_shell_seen,
                "shell decreased at u={u}: was {max_shell_seen}, now {shell}"
            );
            max_shell_seen = shell;
        }
    }

    #[test]
    fn roundtrip_u32_sample() {
        let test_values: Vec<u32> = (0..10000).chain((0..10000).map(|i| i * 100000)).collect();
        for u in test_values {
            let (x, y): (i16, i16) = scatter_triangle(u);
            let back: u32 = scatter_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u32 {u}");
        }
    }

    #[test]
    fn roundtrip_u64_sample() {
        let test_values: Vec<u64> = (0..10000).chain((0..10000).map(|i| i * 100000)).collect();
        for u in test_values {
            let (x, y): (i32, i32) = scatter_triangle(u);
            let back: u64 = scatter_triangle((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }

    #[test]
    fn region_b_points() {
        let region_a_size = 32640u16;
        let mut region_b_points: Vec<(i8, i8)> = Vec::new();
        for u in region_a_size..32896 {
            let (x, y): (i8, i8) = scatter_triangle(u);
            region_b_points.push((x, y));
        }
        assert_eq!(region_b_points.len(), 256);
        for (x, _y) in &region_b_points {
            assert_eq!(*x, i8::MIN, "region B point ({x}, {_y}) should have x=MIN");
        }
    }

    #[test]
    fn ordering_is_scrambled() {
        // Verify that scatter_triangle produces different ordering than spiral_triangle
        // (at least for some shells)
        use crate::bi::spiral_triangle::spiral_triangle;

        let mut different_count = 0;
        for u in 1u16..100 {
            let scatter: (i8, i8) = scatter_triangle(u);
            let spiral: (i8, i8) = spiral_triangle(u);
            if scatter != spiral {
                different_count += 1;
            }
        }
        // Should have many differences due to pseudorandom scrambling
        assert!(
            different_count > 50,
            "scatter and spiral should differ for most indices, only {} differences",
            different_count
        );
    }
}
