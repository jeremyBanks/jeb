//! Built-in bitmap fonts.
use {crate::Font, once_cell::sync::Lazy};

macro_rules! pub_use_lazy_font {
    ($($path:ident as $name:ident),* $(,)?) => {
        $(
            /// ```text
            #[doc = include_str!(concat!(stringify!($path), ".json"))]
            /// ```
            pub static $name: Lazy<Font> = Lazy::new(|| {
                let data = include_bytes!(concat!(stringify!($path), ".png"));
                let meta = include_str!(concat!(stringify!($path), ".json"));
                todo!("{data:?} {meta:?}")
            });
        )*
    };
}

pub_use_lazy_font! {
    micro as MICRO,
    mini as MINI,
    monte as MONTE,
    sixth as SIXTH,
    sky as SKY,
    sugimori as SUGIMORI,
    swiss as SWISS,
}
