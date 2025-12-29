#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between signed and unsigned integer types which preserves the
ordering by magnitude (with each negative value considered lower-magnitude than
its corresponding positive value). Called ZigZag by the Protocol Buffers wire
protocol.
        "#
    };
}
use description;

#[doc = description!()]
pub fn zig_zag<T: ZigZag>(value: T) -> T::Out {
    value.zig_zag()
}

impls! {
    i8: u8;
    i16: u16;
    i64: u64;
    i128: u128;
    isize: usize;
}

#[doc = description!()]
pub trait ZigZag {
    type Out;

    #[doc = description!()]
    fn zig_zag(self) -> Self::Out;
}

macro_rules! impls {
        {$( $signed:ident: $unsigned:ident; )+} => {$(
        impl ZigZag for $signed {
            type Out = $unsigned;

            fn zig_zag(self) -> $unsigned {
                let sign_mask = (self >> (<$signed>::BITS - 1)) as $unsigned;
                 ((self as $unsigned) << 1) ^ sign_mask
            }
        }

        impl ZigZag for $unsigned {
            type Out = $signed;

            fn zig_zag(self) -> $signed {
                let lsb: $signed = (self & 1) as $signed;
                let neg_mask: $signed = -lsb;
                ((self >> 1) as $signed) ^ neg_mask
            }
        }
    )+}
}
use impls;
