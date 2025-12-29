#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between signed and unsigned integer types which preserves the
ordering by magnitude (with each negative value considered lower-magnitude than
its corresponding positive value), as used in the Protocol Buffers wire
protocol.
        "#
    };
}
use description;

#[doc = description!()]
pub fn zigzag<T: Zigzag>(value: T) -> T::Out {
    value.zigzag()
}

impls! {
    i8: u8;
    i16: u16;
    i64: u64;
    i128: u128;
    isize: usize;
}

#[doc = description!()]
pub trait Zigzag {
    type Out;

    #[doc = description!()]
    fn zigzag(self) -> Self::Out;
}

macro_rules! impls {
    {$( $signed:ident: $unsigned:ident; )+} => {}
} use impls;
