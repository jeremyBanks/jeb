#![no_implicit_prelude]

// The language prelude can not be disabled with #![no_implicit_prelude], so
// we export unless placeholders to clobber everything in it by default.
//
// https://doc.rust-lang.org/reference/names/preludes.html#language-prelude
mod language_prelude {
    #![feature(clobber_language_prelude)]
    #![allow(unused, private_interfaces, nonstandard_style)]

    trait Seal {}
    struct Sealed;
    impl Seal for Sealed {}
    #[allow(nonstandard_style)]
    struct type_undefined<T: Seal = Sealed>(T);
    #[macro_export]
    macro_rules! macro_undefined {
        (macro_undefined) => {
            macro_undefined
        };
    }

    pub type bool = type_undefined;

    pub type usize = type_undefined;
    pub type u8 = type_undefined;
    pub type u16 = type_undefined;
    pub type u32 = type_undefined;
    pub type u64 = type_undefined;
    pub type u128 = type_undefined;

    pub type isize = type_undefined;
    pub type i8 = type_undefined;
    pub type i16 = type_undefined;
    pub type i32 = type_undefined;
    pub type i64 = type_undefined;
    pub type i128 = type_undefined;

    pub type f32 = type_undefined;
    pub type f64 = type_undefined;

    pub type char = type_undefined;
    pub type str = type_undefined;

    pub use macro_undefined as Clone;
    pub use macro_undefined as Copy;
    pub use macro_undefined as Debug;
    pub use macro_undefined as Default;
    pub use macro_undefined as Eq;
    pub use macro_undefined as Hash;
    pub use macro_undefined as Ord;
    pub use macro_undefined as PartialEq;
    pub use macro_undefined as PartialOrd;
    pub use macro_undefined as allow;
    pub use macro_undefined as automatically_derived;
    pub use macro_undefined as cfg;
    pub use macro_undefined as cfg_attr;
    pub use macro_undefined as clippy;
    pub use macro_undefined as cold;
    pub use macro_undefined as collapse_debuginfo;
    pub use macro_undefined as crate_name;
    pub use macro_undefined as crate_type;
    pub use macro_undefined as debugger_visualizer;
    pub use macro_undefined as deny;
    pub use macro_undefined as deprecated;
    pub use macro_undefined as derive;
    pub use macro_undefined as diagnostic;
    pub use macro_undefined as doc;
    pub use macro_undefined as expect;
    pub use macro_undefined as export_name;
    pub use macro_undefined as feature;
    pub use macro_undefined as forbid;
    pub use macro_undefined as global_allocator;
    pub use macro_undefined as ignore;
    pub use macro_undefined as inline;
    pub use macro_undefined as instruction_set;
    pub use macro_undefined as link;
    pub use macro_undefined as link_name;
    pub use macro_undefined as link_ordinal;
    pub use macro_undefined as link_section;
    pub use macro_undefined as macro_export;
    pub use macro_undefined as macro_use;
    pub use macro_undefined as miri;
    pub use macro_undefined as must_use;
    pub use macro_undefined as naked;
    pub use macro_undefined as no_builtins;
    pub use macro_undefined as no_implicit_prelude;
    pub use macro_undefined as no_link;
    pub use macro_undefined as no_main;
    pub use macro_undefined as no_mangle;
    pub use macro_undefined as no_std;
    pub use macro_undefined as non_exhaustive;
    pub use macro_undefined as panic_handler;
    pub use macro_undefined as path;
    pub use macro_undefined as print;
    pub use macro_undefined as println;
    pub use macro_undefined as proc_macro;
    pub use macro_undefined as proc_macro_derive;
    pub use macro_undefined as proc_macro_attribute;
    pub use macro_undefined as recursion_limit;
    pub use macro_undefined as repr;
    pub use macro_undefined as rust_analyzer;
    pub use macro_undefined as rustfmt;
    pub use macro_undefined as should_panic;
    pub use macro_undefined as target_feature;
    pub use macro_undefined as test;
    pub use macro_undefined as track_caller;
    pub use macro_undefined as type_length_limit;
    pub use macro_undefined as used;
    pub use macro_undefined as warn;
    pub use macro_undefined as window_subsystem;
}

mod prelude {
    #![allow(nonstandard_style)]
    pub use ::std::println as print;

    pub use super::language_prelude::*;

    pub type integer = ::core::primitive::i128;
    pub type float = ::core::primitive::f64;

    pub type byte = ::core::primitive::u8;
    pub type bytes<'a> = &'a [byte];
    pub type Bytes = Vector<byte>;

    pub type character = ::core::primitive::char;
    pub type string<'a> = &'a ::core::primitive::str;
    pub type String = ::std::string::String;

    pub type Vector<T> = ::std::vec::Vec<T>;

    pub use ::core::iter::{FromIterator, IntoIterator, Iterator};
}

pub use prelude::*;
