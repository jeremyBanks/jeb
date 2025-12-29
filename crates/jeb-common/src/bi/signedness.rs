#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between signed and unsigned integer types which preserves the
ordering and distance relationships of values.
        "#
    };
}
use description;

impls! {
    i8: u8;
    i16: u16;
    i32: u32;
    i64: u64;
    i128: u128;
    isize: usize;
}

use crate::is;

#[doc = description!()]
pub fn signedness<T: Signedness>(value: T) -> T::Out {
    value.signedness()
}

#[doc = description!()]
pub trait Signedness {
    type Out;

    #[doc = description!()]
    fn signedness(self) -> Self::Out;
}

macro_rules! impls {
    {$( $signed:ident: $unsigned:ident; )+} => {$(
        impl Signedness for $signed {
            type Out = $unsigned;

            fn signedness(self) -> $unsigned {
                let high_mask = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self as $unsigned) ^ high_mask
            }
        }

        impl Signedness for $unsigned {
            type Out = $signed;

            fn signedness(self) -> $signed {
                let high_mask = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self ^ high_mask) as $signed
            }
        }
    )+}
}
use impls;
