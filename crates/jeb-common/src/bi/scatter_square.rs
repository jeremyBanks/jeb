#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between between N-bit unsigned integers and pairs of N/2-bit signed
integers where unsigned integers are mapped onto points on successive squares
(Chebyshev L∞ shells), starting at (0, 0), with pseudorandom ordering within
each square.

In other words: as we start at 0 and look at ascending unsigned integers, we'll
first see all of the points whose maximum component magnitude is 0, then all of
the points whose maximum component magnitude is 1, then 2, etc, so we're filling
up concentric squares shells centered at the origin, one shell at a time, but
picking points within each shell using a weakly-pseudorandom ordering.
        "#
    };
}
use description;
impl_with!(u16, i8, u8, 8, scatter_square_u16);
impl_with!(u32, i16, u16, 16, scatter_square_u32);
impl_with!(u64, i32, u32, 32, scatter_square_u64);
#[doc = description!()]
pub fn scatter_square<const SEED: u64, T: ScatterSquare<SEED>>(value: T) -> T::Out {
    value.scatter_square()
}
#[doc = description!()]
pub trait ScatterSquare<const SEED: u64 = 0> {
    type Out;
    #[doc = description!()]
    fn scatter_square(self) -> Self::Out;
}
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
/// Exact integer sqrt floor for u64.
fn isqrt_u64(x: u64) -> u64 {
    let mut op = x;
    let mut res = 0u64;
    let mut one = 1u64 << 62;
    while one > op {
        one >>= 2;
    }
    while one != 0 {
        if op >= res + one {
            op -= res + one;
            res = (res >> 1) + one;
        } else {
            res >>= 1;
        }
        one >>= 2;
    }
    res
}
macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr, $mod:ident) => {
        mod $mod {
            use super::*;
            const W: u32 = $W;
            const MAX_S: i64 = <$S>::MAX as i64;
            const MIN_S: i64 = <$S>::MIN as i64;
            const BANNED: $UB = (1 as $UB) << (W - 1);
            #[inline(always)]
            fn two_w() -> u64 {
                1u64 << W
            }
            #[doc = " Region A size: (2^W - 1)^2"]
            #[inline(always)]
            fn region_a_size() -> u64 {
                let side = two_w() - 1;
                side * side
            }
            #[doc = " Outer layer id: MAX+1 = 2^(W-1)"]
            #[inline(always)]
            fn outer_layer_id() -> u64 {
                (MAX_S as u64) + 1
            }
            #[doc = " Outer ragged size: 2^(W+1) - 1"]
            #[inline(always)]
            fn outer_size() -> u64 {
                (1u64 << (W + 1)) - 1
            }
            #[inline(always)]
            fn base(m: u64) -> u64 {
                if m == 0 { 0 } else { (2 * m - 1) * (2 * m - 1) }
            }
            #[inline(always)]
            fn shell_len(m: u64) -> u64 {
                if m == 0 { 1 } else { 8 * m }
            }
            #[doc = " Map perimeter index j in [0, 8M) to point on shell max(|x|,|y|)=M (M>0)."]
            #[inline(always)]
            fn perimeter_point(m: i64, j: u64) -> (i64, i64) {
                let s = 2u64 * (m as u64);
                if j < s {
                    (-m + j as i64, m)
                } else if j < 2 * s {
                    let jj = j - s;
                    (m, m - jj as i64)
                } else if j < 3 * s {
                    let jj = j - 2 * s;
                    (m - jj as i64, -m)
                } else {
                    let jj = j - 3 * s;
                    (-m, -m + jj as i64)
                }
            }
            #[doc = " Inverse of perimeter_point. Requires shell perimeter point at M>0."]
            #[inline(always)]
            fn perimeter_index(m: i64, x: i64, y: i64) -> u64 {
                let s = 2u64 * (m as u64);
                if y == m && x != m {
                    (x + m) as u64
                } else if x == m && y != -m {
                    s + (m - y) as u64
                } else if y == -m && x != -m {
                    2 * s + (m - x) as u64
                } else {
                    3 * s + (y + m) as u64
                }
            }
            #[inline(always)]
            fn bits_to_signed(bits: $UB) -> $S {
                bits as $S
            }
            #[inline(always)]
            fn signed_to_bits(v: $S) -> $UB {
                v as $UB
            }
            #[doc = " Map k in [0, 2^W - 2] to all $UB except the banned value (MIN bit"]
            #[doc = " pattern)."]
            #[inline(always)]
            fn decompress_skip_banned(k: $UB) -> $UB {
                if k < BANNED { k } else { k.wrapping_add(1) }
            }
            #[doc = " Inverse of decompress_skip_banned for x_bits != banned."]
            #[inline(always)]
            fn compress_skip_banned(x_bits: $UB) -> $UB {
                debug_assert!(x_bits != BANNED);
                if x_bits < BANNED {
                    x_bits
                } else {
                    x_bits.wrapping_sub(1)
                }
            }
            #[doc = " u -> (x,y), total on full $U domain."]
            pub fn to_xy<const SEED: u64>(u_in: $U) -> ($S, $S) {
                let u = u_in as u64;
                let n0 = region_a_size();
                if u < n0 {
                    if u == 0 {
                        return (0 as $S, 0 as $S);
                    }
                    let r = isqrt_u64(u);
                    let m = r.div_ceil(2) as u64;
                    let b = base(m);
                    let t = u - b;
                    let n = shell_len(m);
                    let j = permute_layer(t, n, m, SEED);
                    let (x, y) = perimeter_point(m as i64, j);
                    return (x as $S, y as $S);
                }
                let r = u - n0;
                let n = outer_size();
                debug_assert!(r < n);
                let layer_id = outer_layer_id();
                let rp = permute_layer(r, n, layer_id, SEED);
                let two_w = two_w();
                if rp < two_w {
                    let y_bits = rp as $UB;
                    (MIN_S as $S, bits_to_signed(y_bits))
                } else {
                    let k = (rp - two_w) as $UB;
                    let x_bits = decompress_skip_banned(k);
                    (bits_to_signed(x_bits), MIN_S as $S)
                }
            }
            #[doc = " (x,y) -> u, total on full ($S,$S) domain."]
            pub fn from_xy<const SEED: u64>(x: $S, y: $S) -> $U {
                let n0 = region_a_size();
                if (x as i64) == MIN_S || (y as i64) == MIN_S {
                    let n = outer_size();
                    let layer_id = outer_layer_id();
                    let two_w = two_w();
                    let rp: u64 = if (x as i64) == MIN_S {
                        signed_to_bits(y) as u64
                    } else {
                        debug_assert!((y as i64) == MIN_S);
                        let x_bits = signed_to_bits(x);
                        debug_assert!(x_bits != BANNED);
                        let k = compress_skip_banned(x_bits) as u64;
                        two_w + k
                    };
                    let r = permute_layer_inv(rp, n, layer_id, SEED);
                    let u = n0 + r;
                    return u as $U;
                }
                let xi = x as i64;
                let yi = y as i64;
                let ax = xi.unsigned_abs();
                let ay = yi.unsigned_abs();
                let m = ax.max(ay);
                if m == 0 {
                    return 0 as $U;
                }
                let j = perimeter_index(m as i64, xi, yi);
                let n = shell_len(m);
                let t = permute_layer_inv(j, n, m, SEED);
                let u = base(m) + t;
                u as $U
            }
        }
        impl<const SEED: u64> ScatterSquare<SEED> for $U {
            type Out = ($S, $S);

            #[inline(always)]
            fn scatter_square(self) -> Self::Out {
                $mod::to_xy::<SEED>(self)
            }
        }
        impl<const SEED: u64> ScatterSquare<SEED> for ($S, $S) {
            type Out = $U;

            #[inline(always)]
            fn scatter_square(self) -> Self::Out {
                $mod::from_xy::<SEED>(self.0, self.1)
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
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = scatter_square::<0, _>(u);
            let back: u16 = scatter_square::<0, _>((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u} -> ({x}, {y})");
        }
    }
    #[test]
    fn roundtrip_pair_to_u16() {
        for x in i8::MIN..=i8::MAX {
            for y in i8::MIN..=i8::MAX {
                let u: u16 = scatter_square::<0, _>((x, y));
                let (back_x, back_y): (i8, i8) = scatter_square::<0, _>(u);
                assert_eq!(
                    (x, y),
                    (back_x, back_y),
                    "roundtrip failed for ({x}, {y}) -> {u}",
                );
            }
        }
    }
    #[test]
    fn bijection_coverage_u16() {
        use std::collections::HashSet;
        let mut seen: HashSet<(i8, i8)> = HashSet::new();
        for u in 0u16..=u16::MAX {
            let pair: (i8, i8) = scatter_square::<0, _>(u);
            assert!(seen.insert(pair), "duplicate output for u16 {u}: {pair:?}");
        }
        assert_eq!(seen.len(), 65536);
    }
    #[test]
    fn zero_maps_to_origin() {
        let (x, y): (i8, i8) = scatter_square::<0, _>(0u16);
        assert_eq!((x, y), (0, 0), "0 should map to (0, 0)");
    }
    #[test]
    fn shell_structure() {
        let mut shell_counts: [u32; 129] = [0; 129];
        for u in 0u16..=u16::MAX {
            let (x, y): (i8, i8) = scatter_square::<0, _>(u);
            let shell = (x as i32).abs().max((y as i32).abs()) as usize;
            shell_counts[shell] += 1;
        }
        assert_eq!(shell_counts[0], 1, "shell 0 should have 1 point");
        for m in 1..=127usize {
            let expected = 8 * m as u32;
            assert_eq!(
                shell_counts[m], expected,
                "shell {m} should have {expected} points, got {}",
                shell_counts[m],
            );
        }
        assert_eq!(
            shell_counts[128], 511,
            "shell 128 (ragged) should have 511 points, got {}",
            shell_counts[128],
        );
    }
    #[test]
    fn shells_filled_in_order() {
        let region_a_size = 65025u16;
        let mut max_shell_seen = 0i32;
        for u in 0u16..region_a_size {
            let (x, y): (i8, i8) = scatter_square::<0, _>(u);
            let shell = (x as i32).abs().max((y as i32).abs());
            assert!(
                shell >= max_shell_seen,
                "shell decreased at u={u}: was {max_shell_seen}, now {shell}",
            );
            max_shell_seen = shell;
        }
    }
    #[test]
    fn different_seeds_differ() {
        let mut same_count = 0;
        for u in 1u16..1000 {
            let p0: (i8, i8) = scatter_square::<0, _>(u);
            let p1: (i8, i8) = scatter_square::<12345, _>(u);
            if p0 == p1 {
                same_count += 1;
            }
        }
        assert!(
            same_count < 50,
            "too many matches between seeds: {same_count}/999"
        );
    }
    #[test]
    fn roundtrip_u32_sample() {
        let test_values: Vec<u32> = (0..1000)
            .chain((u32::MAX - 1000)..=u32::MAX)
            .chain((0..10000).map(|i| i * 429496))
            .collect();
        for u in test_values {
            let (x, y): (i16, i16) = scatter_square::<0, _>(u);
            let back: u32 = scatter_square::<0, _>((x, y));
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
            let (x, y): (i32, i32) = scatter_square::<0, _>(u);
            let back: u64 = scatter_square::<0, _>((x, y));
            assert_eq!(u, back, "roundtrip failed for u64 {u}");
        }
    }
    #[test]
    fn region_b_points() {
        let region_a_size = 65025u16;
        let mut region_b_points: Vec<(i8, i8)> = Vec::new();
        for u in region_a_size..=u16::MAX {
            let (x, y): (i8, i8) = scatter_square::<0, _>(u);
            region_b_points.push((x, y));
        }
        assert_eq!(region_b_points.len(), 511);
        for (x, y) in &region_b_points {
            assert!(
                *x == i8::MIN || *y == i8::MIN,
                "region B point ({x}, {y}) doesn't involve MIN",
            );
        }
    }
    #[test]
    fn specific_values() {
        assert_eq!(scatter_square::<0, _>(0u16), (0i8, 0i8));
        let (x, y): (i8, i8) = scatter_square::<0, _>(65024u16);
        let shell = (x as i32).abs().max((y as i32).abs());
        assert_eq!(shell, 127, "last region A point should be in shell 127");
        let mut found_127 = false;
        for u in ((255u16 - 2) * (255u16 - 2))..65025 {
            let (px, py): (i8, i8) = scatter_square::<0, _>(u);
            let s = (px as i32).abs().max((py as i32).abs());
            if s == 127 {
                found_127 = true;
            }
        }
        assert!(found_127, "should find shell 127 points");
    }
}
