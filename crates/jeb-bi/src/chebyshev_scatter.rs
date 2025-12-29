#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between between N-bit unsigned integers and pairs of N/2-bit signed
integers where unsigned integers are mapped onto points on successive Chebyshev
L∞ shells (starting at (0, 0)) with pseudorandom ordering within each shell.

In other words: as we start at 0 and look at ascending unsigned integers, we'll
first see all of the points whose maximum component magnitude is 0, then all of
the points whose maximum component magnitude is 1, then 2, etc, so we're filling
up concentric squares shells centered at the origin, one shell at a time, but
picking points within each shell using a weakly-pseudorandom ordering.
        "#
    };
}
use description;

impl_with!(u16, i8, u8, 8, chebyshev_scatter_u16);
impl_with!(u32, i16, u16, 16, chebyshev_scatter_u32);
impl_with!(u64, i32, u32, 32, chebyshev_scatter_u64);

// Chebyshev (L∞) shell bijections between:
//   u16 <-> (i8,  i8)
//   u32 <-> (i16, i16)
//   u64 <-> (i32, i32)
//   u128 <-> (i64, i64)
//
// Design summary:
// - Region A: enumerate the full symmetric square [-MAX..MAX]^2 in true
//   Chebyshev shells: M = max(|x|, |y|), base(M) = (2M-1)^2, shell size = 8M.
//   Within each shell, apply a per-layer Feistel permutation to remove
//   perimeter bias.
// - Region B: treat all points involving MIN as the “next shell” (M = MAX+1),
//   which is necessarily ragged because +2^(w-1) does not exist in signed
//   twos-complement. The ragged set is exactly: (MIN, y) for all y   plus   (x
//   != MIN, MIN) and is also permuted with the same Feistel+cycle-walk
//   machinery.
// - Total, infallible, bijective, and covers all values of the involved types.
//
// Seed:
// - Seed is a const generic on the trait with a default of 0.
// - `chebyshev(value)` uses SEED=0.
// - `chebyshev_with::<SEED>(value)` lets you choose a compile-time seed.
#[doc = description!()]
pub fn chebyshev_scatter<const SEED: u64, T: ChebyshevScatter<SEED>>(value: T) -> T::Out {
    value.chebyshev_scatter()
}

#[doc = description!()]
pub trait ChebyshevScatter<const SEED: u64 = 0> {
    type Out;

    #[doc = description!()]
    fn chebyshev_scatter(self) -> Self::Out;
}

// -------------------------
// Shared mixing + Feistel core (u64)
// -------------------------

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
    // With seed=0 => mix64(layer_id * KEY_CONST)
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
    let mut d = next_pow2(n);
    let mut bits = d.trailing_zeros(); // log2(d)
    if bits % 2 == 1 {
        d <<= 1;
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
    let mut one = 1u64 << 62; // highest power of four <= 2^64

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

// -------------------------
// Rung generator macro
// -------------------------

macro_rules! impl_with {
    ($U:ty, $S:ty, $UB:ty, $W:expr, $mod:ident) => {
        mod $mod {
            use super::*;

            // Width parameters
            const W: u32 = $W;
            const MAX_S: i64 = <$S>::MAX as i64;
            const MIN_S: i64 = <$S>::MIN as i64;

            // MIN bit pattern in the corresponding unsigned bits type
            const BANNED: $UB = (1 as $UB) << (W - 1);

            #[inline(always)]
            fn two_w() -> u64 {
                1u64 << W
            } // 2^W

            /// Region A size: (2^W - 1)^2
            #[inline(always)]
            fn region_a_size() -> u64 {
                let side = two_w() - 1;
                side * side
            }

            /// Outer layer id: MAX+1 = 2^(W-1)
            #[inline(always)]
            fn outer_layer_id() -> u64 {
                (MAX_S as u64) + 1
            }

            /// Outer ragged size: 2^(W+1) - 1
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

            /// Map perimeter index j in [0, 8M) to point on shell max(|x|,|y|)=M (M>0).
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

            /// Inverse of perimeter_point. Requires shell perimeter point at M>0.
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

            // Bit-pattern casts (two's complement, stable as modulo cast)
            #[inline(always)]
            fn bits_to_signed(bits: $UB) -> $S {
                bits as $S
            }

            #[inline(always)]
            fn signed_to_bits(v: $S) -> $UB {
                v as $UB
            }

            /// Map k in [0, 2^W - 2] to all $UB except the banned value (MIN bit
            /// pattern).
            #[inline(always)]
            fn decompress_skip_banned(k: $UB) -> $UB {
                if k < BANNED { k } else { k.wrapping_add(1) }
            }

            /// Inverse of decompress_skip_banned for x_bits != banned.
            #[inline(always)]
            fn compress_skip_banned(x_bits: $UB) -> $UB {
                debug_assert!(x_bits != BANNED);
                if x_bits < BANNED {
                    x_bits
                } else {
                    x_bits.wrapping_sub(1)
                }
            }

            /// u -> (x,y), total on full $U domain.
            pub fn to_xy<const SEED: u64>(u_in: $U) -> ($S, $S) {
                let u = u_in as u64;
                let n0 = region_a_size();

                // Region A
                if u < n0 {
                    if u == 0 {
                        return (0 as $S, 0 as $S);
                    }

                    let r = isqrt_u64(u);
                    let m = ((r + 1) / 2) as u64; // 1..=MAX

                    let b = base(m);
                    let t = u - b;
                    let n = shell_len(m);

                    let j = permute_layer(t, n, m, SEED);
                    let (x, y) = perimeter_point(m as i64, j);

                    // Guaranteed in [-MAX..MAX], never MIN.
                    return (x as $S, y as $S);
                }

                // Region B: ragged outer layer (M = MAX+1)
                let r = u - n0; // 0..outer_size-1
                let n = outer_size();
                debug_assert!(r < n);

                let layer_id = outer_layer_id();
                let rp = permute_layer(r, n, layer_id, SEED);

                let two_w = two_w(); // 2^W
                if rp < two_w {
                    // (MIN, y) for all y
                    let y_bits = rp as $UB;
                    (MIN_S as $S, bits_to_signed(y_bits))
                } else {
                    // (x != MIN, MIN)
                    let k = (rp - two_w) as $UB; // 0..2^W-2
                    let x_bits = decompress_skip_banned(k);
                    (bits_to_signed(x_bits), MIN_S as $S)
                }
            }

            /// (x,y) -> u, total on full ($S,$S) domain.
            pub fn from_xy<const SEED: u64>(x: $S, y: $S) -> $U {
                let n0 = region_a_size();

                // Region B first: anything touching MIN is outside Region A.
                if (x as i64) == MIN_S || (y as i64) == MIN_S {
                    let n = outer_size();
                    let layer_id = outer_layer_id();
                    let two_w = two_w();

                    let rp: u64 = if (x as i64) == MIN_S {
                        // rp = y_bits in [0, 2^W)
                        signed_to_bits(y) as u64
                    } else {
                        // must have y==MIN and x!=MIN
                        debug_assert!((y as i64) == MIN_S);
                        let x_bits = signed_to_bits(x);
                        debug_assert!(x_bits != BANNED);
                        let k = compress_skip_banned(x_bits) as u64; // 0..2^W-2
                        two_w + k
                    };

                    let r = permute_layer_inv(rp, n, layer_id, SEED);
                    let u = n0 + r;
                    return u as $U;
                }

                // Region A
                let xi = x as i64;
                let yi = y as i64;

                let ax = xi.abs() as u64;
                let ay = yi.abs() as u64;
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

        // Trait impls (both directions)
        impl<const SEED: u64> super::ChebyshevScatter<SEED> for $U {
            type Out = ($S, $S);

            #[inline(always)]
            fn chebyshev_scatter(self) -> Self::Out {
                $mod::to_xy::<SEED>(self)
            }
        }

        impl<const SEED: u64> super::ChebyshevScatter<SEED> for ($S, $S) {
            type Out = $U;

            #[inline(always)]
            fn chebyshev_scatter(self) -> Self::Out {
                $mod::from_xy::<SEED>(self.0, self.1)
            }
        }
    };
} use impl_with;
