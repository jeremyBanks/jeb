//! Toggles between signed and unsigned integer types while preserving the
//! position of values within new type's range, rather than preserving the
//! numerical values (e.g. `0u32`/`u32::MIN` goes to `i32::MIN`, not `0i32`).

use jeb_common::is;

pub fn signedness<T: Signedness>(value: T) -> T::Out {
    value.signedness()
}

impls! {
    i8: u8;
    i16: u16;
    i64: u64;
    i128: u128;
    isize: usize;
}

pub trait Signedness {
    type Out;
    fn signedness(self) -> Self::Out;
}

macro_rules! impls {
    {$( $signed:ident: $unsigned:ident; )+} => {$(
        impl Signedness for $signed {
            type Out = $unsigned;

            fn signedness(self) -> $unsigned {
                let high_bit = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self as $unsigned) ^ high_bit
            }
        }

        impl Signedness for $unsigned {
            type Out = $signed;

            fn signedness(self) -> $signed {
                let high_bit = (is::<$unsigned>(1) << ($unsigned::BITS - 1));
                (self ^ high_bit) as $signed
            }
        }
    )+}
}
use impls;
