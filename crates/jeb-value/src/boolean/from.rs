use crate::Boolean;

/// Error returned when trying to convert a value that is not 0 or 1 to Boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotBooleanError;

impl core::fmt::Display for NotBooleanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value is not 0 or 1")
    }
}

impl std::error::Error for NotBooleanError {}

// [impl jeb-value.boolean.try-from-f32]
impl TryFrom<f32> for Boolean {
    type Error = NotBooleanError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        // Only accept +0.0 (not -0.0) for false, and 1.0 for true
        if value == 0.0 && !value.is_sign_negative() {
            Ok(Boolean::new(false))
        } else if value == 1.0 {
            Ok(Boolean::new(true))
        } else {
            Err(NotBooleanError)
        }
    }
}

// [impl jeb-value.boolean.try-from-f64]
impl TryFrom<f64> for Boolean {
    type Error = NotBooleanError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // Only accept +0.0 (not -0.0) for false, and 1.0 for true
        if value == 0.0 && !value.is_sign_negative() {
            Ok(Boolean::new(false))
        } else if value == 1.0 {
            Ok(Boolean::new(true))
        } else {
            Err(NotBooleanError)
        }
    }
}

macro_rules! impl_try_from_int_for_boolean {
    ($($ty:ty => $rule:literal),* $(,)?) => {
        $(
            // [impl $rule]
            impl TryFrom<$ty> for Boolean {
                type Error = NotBooleanError;

                fn try_from(value: $ty) -> Result<Self, Self::Error> {
                    match value {
                        0 => Ok(Boolean::new(false)),
                        1 => Ok(Boolean::new(true)),
                        _ => Err(NotBooleanError),
                    }
                }
            }
        )*
    };
}

// [impl jeb-value.boolean.try-from-i8]
// [impl jeb-value.boolean.try-from-i16]
// [impl jeb-value.boolean.try-from-i32]
// [impl jeb-value.boolean.try-from-i64]
// [impl jeb-value.boolean.try-from-i128]
// [impl jeb-value.boolean.try-from-u8]
// [impl jeb-value.boolean.try-from-u16]
// [impl jeb-value.boolean.try-from-u32]
// [impl jeb-value.boolean.try-from-u64]
// [impl jeb-value.boolean.try-from-u128]
impl_try_from_int_for_boolean!(
    i8 => "jeb-value.boolean.try-from-i8",
    i16 => "jeb-value.boolean.try-from-i16",
    i32 => "jeb-value.boolean.try-from-i32",
    i64 => "jeb-value.boolean.try-from-i64",
    i128 => "jeb-value.boolean.try-from-i128",
    u8 => "jeb-value.boolean.try-from-u8",
    u16 => "jeb-value.boolean.try-from-u16",
    u32 => "jeb-value.boolean.try-from-u32",
    u64 => "jeb-value.boolean.try-from-u64",
    u128 => "jeb-value.boolean.try-from-u128",
);
