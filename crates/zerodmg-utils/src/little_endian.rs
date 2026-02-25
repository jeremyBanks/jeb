/// Combines two `u8` values into a `u16`
///
/// # Examples
///
/// ```
/// use zerodmg_utils::little_endian::u8s_to_u16;
///
/// assert_eq!(u8s_to_u16(0x01, 0x80), 0x8001);
/// assert_eq!(u8s_to_u16(0x01, 0x00), 0x0001);
/// assert_eq!(u8s_to_u16(0x00, 0x01), 0x0100);
/// ```
pub fn u8s_to_u16(a: u8, b: u8) -> u16 {
    u16::from(a) + (u16::from(b) << 8)
}

/// Splits a `u16` into two `u8`s.
///
/// # Examples
///
/// ```
/// use zerodmg_utils::little_endian::u16_to_u8s;
///
/// assert_eq!(u16_to_u8s(0x8001), (0x01, 0x80));
/// assert_eq!(u16_to_u8s(0x0001), (0x01, 0x00));
/// assert_eq!(u16_to_u8s(0x0100), (0x00, 0x01));
/// ```
pub fn u16_to_u8s(x: u16) -> (u8, u8) {
    (x as u8, (x >> 8) as u8)
}

/// Returns the value of the `offset`th bit in a `u8` `x`.
///
/// # Examples
///
/// ```
/// use zerodmg_utils::little_endian::u8_get_bit;
///
/// assert_eq!(u8_get_bit(0x81, 0), true);
/// assert_eq!(u8_get_bit(0x81, 1), false);
/// assert_eq!(u8_get_bit(0x81, 6), false);
/// assert_eq!(u8_get_bit(0x81, 7), true);
/// ```
///
/// # Panics
///
/// Panics if `offset` is out of bounds (0 - 7).
///
/// ```should_panic
/// use zerodmg_utils::little_endian::u8_get_bit;
///
/// u8_get_bit(0x81, 8);
/// ```
pub fn u8_get_bit(x: u8, offset: u8) -> bool {
    if offset > 7 {
        panic!("bit offset {offset} out of bounds (must be 0-7)");
    }

    (x >> offset) & 1 == 1
}

/// Sets the value of the `offset`th bit in a `u8` `x`.
///
/// # Examples
///
/// ```
/// use zerodmg_utils::little_endian::u8_set_bit;
///
/// let mut x = 0x00;
/// assert_eq!(x, 0x00);
/// u8_set_bit(&mut x, 0, true);
/// assert_eq!(x, 0x01);
/// u8_set_bit(&mut x, 1, false);
/// assert_eq!(x, 0x01);
/// u8_set_bit(&mut x, 6, false);
/// assert_eq!(x, 0x01);
/// u8_set_bit(&mut x, 7, true);
/// assert_eq!(x, 0x81);
/// ```
///
/// # Panics
///
/// Panics if `offset` is out of bounds (0 - 7).
///
/// ```should_panic
/// use zerodmg_utils::little_endian::u8_set_bit;
///
/// let mut x = 0x00;
/// u8_set_bit(&mut x, 8, true);
/// ```
pub fn u8_set_bit(x: &mut u8, offset: u8, value: bool) {
    if offset > 7 {
        panic!("bit offset {offset} out of bounds (must be 0-7)");
    }

    let mask = 1 << offset;
    if value {
        *x |= mask;
    } else {
        *x &= !mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::proptest::prelude::*;

    proptest! {
        /// Roundtrip: u8s -> u16 -> u8s
        #[test]
        fn u8s_roundtrip(a: u8, b: u8) {
            let combined = u8s_to_u16(a, b);
            let (a2, b2) = u16_to_u8s(combined);
            prop_assert_eq!(a, a2);
            prop_assert_eq!(b, b2);
        }

        /// Roundtrip: u16 -> u8s -> u16
        #[test]
        fn u16_roundtrip(x: u16) {
            let (a, b) = u16_to_u8s(x);
            let x2 = u8s_to_u16(a, b);
            prop_assert_eq!(x, x2);
        }

        /// Get-Set roundtrip: setting a bit then getting it returns the set value
        #[test]
        fn bit_set_get_roundtrip(initial: u8, offset in 0u8..8, value: bool) {
            let mut x = initial;
            u8_set_bit(&mut x, offset, value);
            prop_assert_eq!(u8_get_bit(x, offset), value);
        }

        /// Setting a bit only affects that bit
        #[test]
        fn bit_set_isolation(initial: u8, offset in 0u8..8, value: bool) {
            let mut x = initial;
            u8_set_bit(&mut x, offset, value);
            
            // Check all other bits are unchanged
            for other in 0u8..8 {
                if other != offset {
                    prop_assert_eq!(
                        u8_get_bit(x, other),
                        u8_get_bit(initial, other),
                        "bit {} changed when setting bit {}", other, offset
                    );
                }
            }
        }

        /// Setting a bit twice to the same value is idempotent
        #[test]
        fn bit_set_idempotent(initial: u8, offset in 0u8..8, value: bool) {
            let mut x1 = initial;
            u8_set_bit(&mut x1, offset, value);
            
            let mut x2 = x1;
            u8_set_bit(&mut x2, offset, value);
            
            prop_assert_eq!(x1, x2);
        }
    }
}
