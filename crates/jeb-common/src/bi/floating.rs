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
    f32 : u32; f64 : u64;
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
    {$($float:ident : $uint:ident;)+} => {
        $(impl Floating for $float { type Out = $uint; fn floating(self) -> $uint { let
        bits = self.to_bits(); let sign_bit = is::<$uint > (1) << ($uint ::BITS - 1); if
        (bits & sign_bit) != 0 { ! bits } else { bits ^ sign_bit } } } impl Floating for
        $uint { type Out = $float; fn floating(self) -> $float { let sign_bit =
        is::<$uint > (1) << ($uint ::BITS - 1); let bits = if (self & sign_bit) != 0 {
        self ^ sign_bit } else { ! self }; $float ::from_bits(bits) } })+
    };
}
use impls;
#[cfg(test)]
mod tests {
    use super::*;
    const F64_QNAN: f64 = f64::NAN;
    const F64_QNAN_NEG: u64 = 0xFFF8_0000_0000_0000;
    const F64_SNAN: u64 = 0x7FF0_0000_0000_0001;
    const F64_SNAN_NEG: u64 = 0xFFF0_0000_0000_0001;
    const F64_QNAN_PAYLOAD: u64 = 0x7FF8_0000_0000_1234;
    const F32_QNAN_NEG: u32 = 0xFFC0_0000;
    const F32_SNAN: u32 = 0x7F80_0001;
    const F32_SNAN_NEG: u32 = 0xFF80_0001;
    const F32_QNAN_PAYLOAD: u32 = 0x7FC0_1234;
    const F64_SUBNORMAL_MIN: f64 = 5e-324;
    const F64_SUBNORMAL_MAX: u64 = 0x000F_FFFF_FFFF_FFFF;
    const F32_SUBNORMAL_MIN: f32 = 1e-45;
    const F32_SUBNORMAL_MAX: u32 = 0x007F_FFFF;
    #[test]
    fn test_f64_ordering_preservation() {
        let test_values = vec![
            f64::from_bits(F64_QNAN_NEG),
            f64::from_bits(F64_SNAN_NEG),
            f64::NEG_INFINITY,
            -f64::MAX,
            -1e100,
            -100.0,
            -1.0,
            -0.5,
            -f64::MIN_POSITIVE,
            -f64::from_bits(F64_SUBNORMAL_MAX),
            -F64_SUBNORMAL_MIN,
            -0.0,
            0.0,
            F64_SUBNORMAL_MIN,
            f64::from_bits(F64_SUBNORMAL_MAX),
            f64::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e100,
            f64::MAX,
            f64::INFINITY,
            f64::from_bits(F64_SNAN),
            F64_QNAN,
            f64::from_bits(F64_QNAN_PAYLOAD),
        ];
        for i in 0..test_values.len() - 1 {
            let a = test_values[i];
            let b = test_values[i + 1];
            let a_enc = floating(a);
            let b_enc = floating(b);
            assert!(
                a.total_cmp(&b).is_lt(),
                "Test data should be ordered: {} < {}",
                a,
                b
            );
            assert!(
                a_enc < b_enc,
                "Encoded values should preserve order: {} < {} (from {} < {})",
                a_enc,
                b_enc,
                a,
                b
            );
        }
    }
    #[test]
    fn test_f64_bijection() {
        let test_values = vec![
            f64::from_bits(F64_QNAN_NEG),
            f64::from_bits(F64_SNAN_NEG),
            F64_QNAN,
            f64::from_bits(F64_SNAN),
            f64::from_bits(F64_QNAN_PAYLOAD),
            f64::NEG_INFINITY,
            f64::INFINITY,
            -f64::MAX,
            -1e100,
            -100.0,
            -1.0,
            -0.5,
            -f64::MIN_POSITIVE,
            f64::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e100,
            f64::MAX,
            -f64::from_bits(F64_SUBNORMAL_MAX),
            -F64_SUBNORMAL_MIN,
            F64_SUBNORMAL_MIN,
            f64::from_bits(F64_SUBNORMAL_MAX),
            -0.0,
            0.0,
        ];
        for &val in &test_values {
            let encoded = floating(val);
            let decoded: f64 = floating(encoded);
            assert_eq!(
                val.to_bits(),
                decoded.to_bits(),
                "Round-trip failed for {}: encoded={}, decoded={}",
                val,
                encoded,
                decoded
            );
        }
    }
    #[test]
    fn test_f32_ordering_preservation() {
        let test_values = vec![
            f32::from_bits(F32_QNAN_NEG),
            f32::from_bits(F32_SNAN_NEG),
            f32::NEG_INFINITY,
            -f32::MAX,
            -1e30,
            -100.0,
            -1.0,
            -0.5,
            -f32::MIN_POSITIVE,
            -f32::from_bits(F32_SUBNORMAL_MAX),
            -F32_SUBNORMAL_MIN,
            -0.0,
            0.0,
            F32_SUBNORMAL_MIN,
            f32::from_bits(F32_SUBNORMAL_MAX),
            f32::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e30,
            f32::MAX,
            f32::INFINITY,
            f32::from_bits(F32_SNAN),
            f32::NAN,
            f32::from_bits(F32_QNAN_PAYLOAD),
        ];
        for i in 0..test_values.len() - 1 {
            let a = test_values[i];
            let b = test_values[i + 1];
            let a_enc = floating(a);
            let b_enc = floating(b);
            assert!(
                a.total_cmp(&b).is_lt(),
                "Test data should be ordered: {} < {}",
                a,
                b
            );
            assert!(
                a_enc < b_enc,
                "Encoded values should preserve order: {} < {} (from {} < {})",
                a_enc,
                b_enc,
                a,
                b
            );
        }
    }
    #[test]
    fn test_f32_bijection() {
        let test_values = vec![
            f32::from_bits(F32_QNAN_NEG),
            f32::from_bits(F32_SNAN_NEG),
            f32::NAN,
            f32::from_bits(F32_SNAN),
            f32::from_bits(F32_QNAN_PAYLOAD),
            f32::NEG_INFINITY,
            f32::INFINITY,
            -f32::MAX,
            -1e30,
            -100.0,
            -1.0,
            -0.5,
            -f32::MIN_POSITIVE,
            f32::MIN_POSITIVE,
            0.5,
            1.0,
            100.0,
            1e30,
            f32::MAX,
            -f32::from_bits(F32_SUBNORMAL_MAX),
            -F32_SUBNORMAL_MIN,
            F32_SUBNORMAL_MIN,
            f32::from_bits(F32_SUBNORMAL_MAX),
            -0.0,
            0.0,
        ];
        for &val in &test_values {
            let encoded = floating(val);
            let decoded: f32 = floating(encoded);
            assert_eq!(
                val.to_bits(),
                decoded.to_bits(),
                "Round-trip failed for {}: encoded={}, decoded={}",
                val,
                encoded,
                decoded
            );
        }
    }
    #[test]
    fn test_negative_zero_vs_positive_zero() {
        let neg_zero = -0.0f64;
        let pos_zero = 0.0f64;
        let neg_enc = floating(neg_zero);
        let pos_enc = floating(pos_zero);
        assert_ne!(neg_enc, pos_enc);
        assert!(neg_enc < pos_enc);
    }
}
