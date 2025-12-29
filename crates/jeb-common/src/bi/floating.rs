#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between floating point and unsigned integer types which preserves
the ordering of values (as defined by IEEE 754-2008).
        "#
    };
}
use description;

impls! {
    f32: u32;
    f64: u64;
}

use crate::is;

#[doc = description!()]
pub fn floating<T: Floating>(value: T) -> T::Out {
    value.floating()
}

#[doc = description!()]
pub trait Floating {
    type Out;

    #[doc = description!()]
    fn floating(self) -> Self::Out;
}

macro_rules! impls {
    {$( $float:ident: $uint:ident; )+} => {$(
        impl Floating for $float {
            type Out = $uint;

            fn floating(self) -> $uint {
                let bits = self.to_bits();
                let sign_bit = is::<$uint>(1) << ($uint::BITS - 1);

                if (bits & sign_bit) != 0 {
                    // Negative number: flip all bits
                    !bits
                } else {
                    // Non-negative number: flip only the sign bit
                    bits ^ sign_bit
                }
            }
        }

        impl Floating for $uint {
            type Out = $float;

            fn floating(self) -> $float {
                let sign_bit = is::<$uint>(1) << ($uint::BITS - 1);

                let bits = if (self & sign_bit) != 0 {
                    // Was non-negative: flip only the sign bit
                    self ^ sign_bit
                } else {
                    // Was negative: flip all bits
                    !self
                };

                $float::from_bits(bits)
            }
        }
    )+}
}
use impls;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_ordering_preservation() {
        let test_values = vec![
            f64::NEG_INFINITY,
            -1e100,
            -100.0,
            -1.0,
            -0.5,
            -f64::MIN_POSITIVE,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e100,
            f64::INFINITY,
        ];

        // Test that ordering is preserved using IEEE 754-2008 totalOrder
        for i in 0..test_values.len() - 1 {
            let a = test_values[i];
            let b = test_values[i + 1];
            let a_enc = floating(a);
            let b_enc = floating(b);

            assert!(a.total_cmp(&b).is_lt(), "Test data should be ordered: {} < {}", a, b);
            assert!(a_enc < b_enc, "Encoded values should preserve order: {} < {} (from {} < {})",
                    a_enc, b_enc, a, b);
        }
    }

    #[test]
    fn test_f64_bijection() {
        let test_values = vec![
            f64::NEG_INFINITY,
            -1e100,
            -100.0,
            -1.0,
            -0.5,
            -f64::MIN_POSITIVE,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e100,
            f64::INFINITY,
        ];

        for &val in &test_values {
            let encoded = floating(val);
            let decoded: f64 = floating(encoded);

            // Use to_bits for comparison to handle -0.0 vs 0.0
            assert_eq!(val.to_bits(), decoded.to_bits(),
                      "Round-trip failed for {}: encoded={}, decoded={}",
                      val, encoded, decoded);
        }
    }

    #[test]
    fn test_f32_ordering_preservation() {
        let test_values = vec![
            f32::NEG_INFINITY,
            -1e30,
            -100.0,
            -1.0,
            -0.5,
            -f32::MIN_POSITIVE,
            -0.0,
            0.0,
            f32::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e30,
            f32::INFINITY,
        ];

        // Test that ordering is preserved using IEEE 754-2008 totalOrder
        for i in 0..test_values.len() - 1 {
            let a = test_values[i];
            let b = test_values[i + 1];
            let a_enc = floating(a);
            let b_enc = floating(b);

            assert!(a.total_cmp(&b).is_lt(), "Test data should be ordered: {} < {}", a, b);
            assert!(a_enc < b_enc, "Encoded values should preserve order: {} < {} (from {} < {})",
                    a_enc, b_enc, a, b);
        }
    }

    #[test]
    fn test_f32_bijection() {
        let test_values = vec![
            f32::NEG_INFINITY,
            -1e30,
            -100.0,
            -1.0,
            -0.5,
            -f32::MIN_POSITIVE,
            -0.0,
            0.0,
            f32::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e30,
            f32::INFINITY,
        ];

        for &val in &test_values {
            let encoded = floating(val);
            let decoded: f32 = floating(encoded);

            assert_eq!(val.to_bits(), decoded.to_bits(),
                      "Round-trip failed for {}: encoded={}, decoded={}",
                      val, encoded, decoded);
        }
    }

    #[test]
    fn test_negative_zero_vs_positive_zero() {
        let neg_zero = -0.0f64;
        let pos_zero = 0.0f64;

        let neg_enc = floating(neg_zero);
        let pos_enc = floating(pos_zero);

        // -0.0 should encode to a different value than +0.0
        assert_ne!(neg_enc, pos_enc);

        // -0.0 should encode to a smaller value than +0.0 (preserving IEEE 754-2008 totalOrder)
        assert!(neg_enc < pos_enc);
    }
}
