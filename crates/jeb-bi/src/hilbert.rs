use jeb_common::is;

pub fn hilbert<T: Hilbert>(value: T) -> T::Out {
    value.hilbert()
}

impls! {
    u16: (u8, u8);

}

pub trait Hilbert {
    type Out;
    fn hilbert(self) -> Self::Out;
}

macro_rules! impls {
    {
        $( $full:ident: ($half1:ident, $half2:ident); )+
    } => {
        $(
            fn __() {
                _ = |assert: $half1| -> $half2 { assert };
            }

            impl Hilbert for $full {
                type Out = ($half1, $half2);

                fn hilbert(self) -> ($half1, $half2) {
                    // XXX: order?!
                    ::fast_hilbert::h2xy(self, 1)
                }
            }
        )+
    }
}
use impls;
