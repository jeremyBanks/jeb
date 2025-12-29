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
