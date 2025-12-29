#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between between N-bit unsigned integers and pairs of N/2-bit unsigned
integers which preserves locality through an order-N/2 Hilbert curve.
        "#
    };
}
use description;

#[doc = description!()]
pub fn hilbert<T: Hilbert>(value: T) -> T::Out {
    value.hilbert()
}

impls! {
    u16: (u8, u8);
    u32: (u16, u16);
    u64: (u32, u32);
    u128: (u64, u64);
}

#[doc = description!()]
pub trait Hilbert {
    type Out;

    #[doc = description!()]
    fn hilbert(self) -> Self::Out;
}

macro_rules! impls {
    {
        $( $full:ident: ($half1:ident, $half2:ident); )+
    } => {
        $(
            impl Hilbert for $full {
                type Out = ($half1, $half2);

                fn hilbert(self) -> ($half1, $half2) {
                    _ = |assert: $half1| -> $half2 { assert };

                    ::fast_hilbert::h2xy(self, $half1::BITS.try_into().unwrap())
                }
            }

            impl Hilbert for ($half1, $half2) {
                type Out = $full;

                fn hilbert(self) -> $full {
                    let (x, y) = self;
                    ::fast_hilbert::xy2h(x, y, $half1::BITS.try_into().unwrap())
                }
            }
        )+
    }
}
use impls;
