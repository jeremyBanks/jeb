use crate::{Bytes, String};

impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<String> for Bytes {
    fn from(value: String) -> Self {
        Bytes(value.into_inner().into_bytes())
    }
}

// [impl jeb-value.bytes.from-unit]
impl From<()> for Bytes {
    fn from(_: ()) -> Self {
        Bytes(Vec::new())
    }
}

// [impl jeb-value.bytes.from-bool]
impl From<bool> for Bytes {
    fn from(value: bool) -> Self {
        Bytes(vec![if value { 0x01 } else { 0x00 }])
    }
}

// [impl jeb-value.bytes.from-char]
impl From<char> for Bytes {
    fn from(value: char) -> Self {
        Bytes((value as u32).to_be_bytes().to_vec())
    }
}

// [impl jeb-value.bytes.from-f32]
impl From<f32> for Bytes {
    fn from(value: f32) -> Self {
        Bytes(value.to_be_bytes().to_vec())
    }
}

// [impl jeb-value.bytes.from-f64]
impl From<f64> for Bytes {
    fn from(value: f64) -> Self {
        Bytes(value.to_be_bytes().to_vec())
    }
}

macro_rules! impl_from_int_for_bytes {
    ($($ty:ty => $rule:literal),* $(,)?) => {
        $(
            // [impl $rule]
            impl From<$ty> for Bytes {
                fn from(value: $ty) -> Self {
                    Bytes(value.to_be_bytes().to_vec())
                }
            }
        )*
    };
}

// [impl jeb-value.bytes.from-i8]
// [impl jeb-value.bytes.from-i16]
// [impl jeb-value.bytes.from-i32]
// [impl jeb-value.bytes.from-i64]
// [impl jeb-value.bytes.from-i128]
// [impl jeb-value.bytes.from-u8]
// [impl jeb-value.bytes.from-u16]
// [impl jeb-value.bytes.from-u32]
// [impl jeb-value.bytes.from-u64]
// [impl jeb-value.bytes.from-u128]
impl_from_int_for_bytes!(
    i8 => "jeb-value.bytes.from-i8",
    i16 => "jeb-value.bytes.from-i16",
    i32 => "jeb-value.bytes.from-i32",
    i64 => "jeb-value.bytes.from-i64",
    i128 => "jeb-value.bytes.from-i128",
    u8 => "jeb-value.bytes.from-u8",
    u16 => "jeb-value.bytes.from-u16",
    u32 => "jeb-value.bytes.from-u32",
    u64 => "jeb-value.bytes.from-u64",
    u128 => "jeb-value.bytes.from-u128",
);

// [impl jeb-value.bytes.from-iterator]
impl FromIterator<u8> for Bytes {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Bytes(iter.into_iter().collect())
    }
}

// [impl jeb-value.bytes.from-slice-iterator]
impl<'a> FromIterator<&'a [u8]> for Bytes {
    fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
        Bytes(iter.into_iter().flatten().copied().collect())
    }
}
