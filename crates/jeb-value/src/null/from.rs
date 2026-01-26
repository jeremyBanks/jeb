use {
    crate::Null,
    ordermap::OrderMap,
};

/// Error returned when trying to convert a non-zero value to Null.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotNullError;

impl core::fmt::Display for NotNullError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value is not null/zero")
    }
}

impl std::error::Error for NotNullError {}

// [impl jeb-value.null.try-from-bool]
impl TryFrom<bool> for Null {
    type Error = NotNullError;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if !value {
            Ok(Null::new())
        } else {
            Err(NotNullError)
        }
    }
}

// [impl jeb-value.null.try-from-f32]
impl TryFrom<f32> for Null {
    type Error = NotNullError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        // Only accept +0.0 (not -0.0)
        if value == 0.0 && !value.is_sign_negative() {
            Ok(Null::new())
        } else {
            Err(NotNullError)
        }
    }
}

// [impl jeb-value.null.try-from-f64]
impl TryFrom<f64> for Null {
    type Error = NotNullError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // Only accept +0.0 (not -0.0)
        if value == 0.0 && !value.is_sign_negative() {
            Ok(Null::new())
        } else {
            Err(NotNullError)
        }
    }
}

macro_rules! impl_try_from_int_for_null {
    ($($ty:ty => $rule:literal),* $(,)?) => {
        $(
            // [impl $rule]
            impl TryFrom<$ty> for Null {
                type Error = NotNullError;

                fn try_from(value: $ty) -> Result<Self, Self::Error> {
                    if value == 0 {
                        Ok(Null::new())
                    } else {
                        Err(NotNullError)
                    }
                }
            }
        )*
    };
}

// [impl jeb-value.null.try-from-i8]
// [impl jeb-value.null.try-from-i16]
// [impl jeb-value.null.try-from-i32]
// [impl jeb-value.null.try-from-i64]
// [impl jeb-value.null.try-from-i128]
// [impl jeb-value.null.try-from-u8]
// [impl jeb-value.null.try-from-u16]
// [impl jeb-value.null.try-from-u32]
// [impl jeb-value.null.try-from-u64]
// [impl jeb-value.null.try-from-u128]
impl_try_from_int_for_null!(
    i8 => "jeb-value.null.try-from-i8",
    i16 => "jeb-value.null.try-from-i16",
    i32 => "jeb-value.null.try-from-i32",
    i64 => "jeb-value.null.try-from-i64",
    i128 => "jeb-value.null.try-from-i128",
    u8 => "jeb-value.null.try-from-u8",
    u16 => "jeb-value.null.try-from-u16",
    u32 => "jeb-value.null.try-from-u32",
    u64 => "jeb-value.null.try-from-u64",
    u128 => "jeb-value.null.try-from-u128",
);

// [impl jeb-value.null.try-from-vec]
impl<T> TryFrom<Vec<T>> for Null {
    type Error = NotNullError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(Null::new())
        } else {
            Err(NotNullError)
        }
    }
}

// [impl jeb-value.null.try-from-ordermap]
impl<K, V> TryFrom<OrderMap<K, V>> for Null {
    type Error = NotNullError;

    fn try_from(value: OrderMap<K, V>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(Null::new())
        } else {
            Err(NotNullError)
        }
    }
}

// Into implementations (From<Null> for T)

// [impl jeb-value.null.into-bool]
impl From<Null> for bool {
    fn from(_value: Null) -> Self {
        false
    }
}

// [impl jeb-value.null.into-f32]
impl From<Null> for f32 {
    fn from(_value: Null) -> Self {
        0.0
    }
}

// [impl jeb-value.null.into-f64]
impl From<Null> for f64 {
    fn from(_value: Null) -> Self {
        0.0
    }
}

macro_rules! impl_from_null_for_int {
    ($($ty:ty => $rule:literal),* $(,)?) => {
        $(
            // [impl $rule]
            impl From<Null> for $ty {
                fn from(_value: Null) -> Self {
                    0
                }
            }
        )*
    };
}

// [impl jeb-value.null.into-i8]
// [impl jeb-value.null.into-i16]
// [impl jeb-value.null.into-i32]
// [impl jeb-value.null.into-i64]
// [impl jeb-value.null.into-i128]
// [impl jeb-value.null.into-u8]
// [impl jeb-value.null.into-u16]
// [impl jeb-value.null.into-u32]
// [impl jeb-value.null.into-u64]
// [impl jeb-value.null.into-u128]
impl_from_null_for_int!(
    i8 => "jeb-value.null.into-i8",
    i16 => "jeb-value.null.into-i16",
    i32 => "jeb-value.null.into-i32",
    i64 => "jeb-value.null.into-i64",
    i128 => "jeb-value.null.into-i128",
    u8 => "jeb-value.null.into-u8",
    u16 => "jeb-value.null.into-u16",
    u32 => "jeb-value.null.into-u32",
    u64 => "jeb-value.null.into-u64",
    u128 => "jeb-value.null.into-u128",
);

// [impl jeb-value.null.into-vec]
impl<T> From<Null> for Vec<T> {
    fn from(_value: Null) -> Self {
        Vec::new()
    }
}

// [impl jeb-value.null.into-ordermap]
impl<K, V> From<Null> for OrderMap<K, V> {
    fn from(_value: Null) -> Self {
        OrderMap::new()
    }
}
