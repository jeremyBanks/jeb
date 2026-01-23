#![doc = description!()]

macro_rules! description {
    () => {
        r#"
Bijection between N-bit unsigned integers and pairs of N/2-bit unsigned integers
using the Z-order curve (Morton code). This preserves locality, but less effectively
than the Hilbert curve. It is computed by interleaving the bits of the two coordinates.
        "#
    };
}
// spell-checker: disable

use description;

#[doc = description!()]
pub fn z_order<T: ZOrder>(value: T) -> T::Out {
    value.z_order()
}

// Manual implementation to avoid `paste` dependency

impl ZOrder for u16 {
    type Out = (u8, u8);

    fn z_order(self) -> (u8, u8) {
        deinterleave_bits_u16(self)
    }
}
impl ZOrder for (u8, u8) {
    type Out = u16;

    fn z_order(self) -> u16 {
        interleave_bits_u16(self.0, self.1)
    }
}

impl ZOrder for u32 {
    type Out = (u16, u16);

    fn z_order(self) -> (u16, u16) {
        deinterleave_bits_u32(self)
    }
}
impl ZOrder for (u16, u16) {
    type Out = u32;

    fn z_order(self) -> u32 {
        interleave_bits_u32(self.0, self.1)
    }
}

impl ZOrder for u64 {
    type Out = (u32, u32);

    fn z_order(self) -> (u32, u32) {
        deinterleave_bits_u64(self)
    }
}
impl ZOrder for (u32, u32) {
    type Out = u64;

    fn z_order(self) -> u64 {
        interleave_bits_u64(self.0, self.1)
    }
}

impl ZOrder for u128 {
    type Out = (u64, u64);

    fn z_order(self) -> (u64, u64) {
        deinterleave_bits_u128(self)
    }
}
impl ZOrder for (u64, u64) {
    type Out = u128;

    fn z_order(self) -> u128 {
        interleave_bits_u128(self.0, self.1)
    }
}

#[doc = description!()]
pub trait ZOrder {
    type Out;
    #[doc = description!()]
    fn z_order(self) -> Self::Out;
}

/// Interleaves bits of x and y.
/// x becomes the even bits (0, 2, 4...)
/// y becomes the odd bits (1, 3, 5...)
fn interleave_bits_u16(x: u8, y: u8) -> u16 {
    let mut x = x as u16;
    let mut y = y as u16;

    x = (x | (x << 4)) & 0x0F0F;
    x = (x | (x << 2)) & 0x3333;
    x = (x | (x << 1)) & 0x5555;

    y = (y | (y << 4)) & 0x0F0F;
    y = (y | (y << 2)) & 0x3333;
    y = (y | (y << 1)) & 0x5555;

    x | (y << 1)
}

fn deinterleave_bits_u16(n: u16) -> (u8, u8) {
    let mut x = n & 0x5555;
    let mut y = (n >> 1) & 0x5555;

    x = (x | (x >> 1)) & 0x3333;
    x = (x | (x >> 2)) & 0x0F0F;
    x = (x | (x >> 4)) & 0x00FF;

    y = (y | (y >> 1)) & 0x3333;
    y = (y | (y >> 2)) & 0x0F0F;
    y = (y | (y >> 4)) & 0x00FF;

    (x as u8, y as u8)
}

fn interleave_bits_u32(x: u16, y: u16) -> u32 {
    let mut x = x as u32;
    let mut y = y as u32;

    x = (x | (x << 8)) & 0x00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F;
    x = (x | (x << 2)) & 0x33333333;
    x = (x | (x << 1)) & 0x55555555;

    y = (y | (y << 8)) & 0x00FF00FF;
    y = (y | (y << 4)) & 0x0F0F0F0F;
    y = (y | (y << 2)) & 0x33333333;
    y = (y | (y << 1)) & 0x55555555;

    x | (y << 1)
}

fn deinterleave_bits_u32(n: u32) -> (u16, u16) {
    let mut x = n & 0x55555555;
    let mut y = (n >> 1) & 0x55555555;

    x = (x | (x >> 1)) & 0x33333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF;

    y = (y | (y >> 1)) & 0x33333333;
    y = (y | (y >> 2)) & 0x0F0F0F0F;
    y = (y | (y >> 4)) & 0x00FF00FF;
    y = (y | (y >> 8)) & 0x0000FFFF;

    (x as u16, y as u16)
}

fn interleave_bits_u64(x: u32, y: u32) -> u64 {
    let mut x = x as u64;
    let mut y = y as u64;

    x = (x | (x << 16)) & 0x0000FFFF0000FFFF;
    x = (x | (x << 8)) & 0x00FF00FF00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;
    x = (x | (x << 2)) & 0x3333333333333333;
    x = (x | (x << 1)) & 0x5555555555555555;

    y = (y | (y << 16)) & 0x0000FFFF0000FFFF;
    y = (y | (y << 8)) & 0x00FF00FF00FF00FF;
    y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F;
    y = (y | (y << 2)) & 0x3333333333333333;
    y = (y | (y << 1)) & 0x5555555555555555;

    x | (y << 1)
}

fn deinterleave_bits_u64(n: u64) -> (u32, u32) {
    let mut x = n & 0x5555555555555555;
    let mut y = (n >> 1) & 0x5555555555555555;

    x = (x | (x >> 1)) & 0x3333333333333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF0000FFFF;
    x = (x | (x >> 16)) & 0x00000000FFFFFFFF;

    y = (y | (y >> 1)) & 0x3333333333333333;
    y = (y | (y >> 2)) & 0x0F0F0F0F0F0F0F0F;
    y = (y | (y >> 4)) & 0x00FF00FF00FF00FF;
    y = (y | (y >> 8)) & 0x0000FFFF0000FFFF;
    y = (y | (y >> 16)) & 0x00000000FFFFFFFF;

    (x as u32, y as u32)
}

fn interleave_bits_u128(x: u64, y: u64) -> u128 {
    let mut x = x as u128;
    let mut y = y as u128;

    x = (x | (x << 32)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
    x = (x | (x << 16)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
    x = (x | (x << 8)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
    x = (x | (x << 2)) & 0x33333333333333333333333333333333;
    x = (x | (x << 1)) & 0x55555555555555555555555555555555;

    y = (y | (y << 32)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
    y = (y | (y << 16)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
    y = (y | (y << 8)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
    y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
    y = (y | (y << 2)) & 0x33333333333333333333333333333333;
    y = (y | (y << 1)) & 0x55555555555555555555555555555555;

    x | (y << 1)
}

fn deinterleave_bits_u128(n: u128) -> (u64, u64) {
    let mut x = n & 0x55555555555555555555555555555555;
    let mut y = (n >> 1) & 0x55555555555555555555555555555555;

    x = (x | (x >> 1)) & 0x33333333333333333333333333333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
    x = (x | (x >> 16)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
    x = (x | (x >> 32)) & 0x0000000000000000FFFFFFFFFFFFFFFF;

    y = (y | (y >> 1)) & 0x33333333333333333333333333333333;
    y = (y | (y >> 2)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
    y = (y | (y >> 4)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
    y = (y | (y >> 8)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
    y = (y | (y >> 16)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
    y = (y | (y >> 32)) & 0x0000000000000000FFFFFFFFFFFFFFFF;

    (x as u64, y as u64)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        proptest::prelude::*,
    };

    #[test]
    fn roundtrip_u16_sample() {
        for u in 0u16..=u16::MAX {
            let (x, y): (u8, u8) = z_order(u);
            let back: u16 = z_order((x, y));
            assert_eq!(u, back, "roundtrip failed for u16 {u}");
        }
    }

    #[test]
    fn roundtrip_pair_sample() {
        for x in 0u8..=u8::MAX {
            for y in 0u8..=u8::MAX {
                let u: u16 = z_order((x, y));
                let (back_x, back_y): (u8, u8) = z_order(u);
                assert_eq!((x, y), (back_x, back_y), "roundtrip failed for ({x}, {y})");
            }
        }
    }

    proptest! {
        #[test]
        fn prop_roundtrip_u16(u in proptest::num::u16::ANY) {
            let (x, y): (u8, u8) = z_order(u);
            let back: u16 = z_order((x, y));
            prop_assert_eq!(u, back);
        }

        #[test]
        fn prop_roundtrip_u32(u in proptest::num::u32::ANY) {
            let (x, y): (u16, u16) = z_order(u);
            let back: u32 = z_order((x, y));
            prop_assert_eq!(u, back);
        }

        #[test]
        fn prop_roundtrip_u64(u in proptest::num::u64::ANY) {
            let (x, y): (u32, u32) = z_order(u);
            let back: u64 = z_order((x, y));
            prop_assert_eq!(u, back);
        }

        #[test]
        fn prop_roundtrip_u128(u in proptest::num::u128::ANY) {
            let (x, y): (u64, u64) = z_order(u);
            let back: u128 = z_order((x, y));
            prop_assert_eq!(u, back);
        }
    }
}
