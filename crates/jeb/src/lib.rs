#![warn(
    clippy::std_instead_of_core,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::allow_attributes,
    clippy::arbitrary_source_item_ordering
)]
#![expect(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::redundant_else,
    clippy::needless_continue,
    clippy::manual_assert,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::default_constructed_unit_structs,
    clippy::too_long_first_doc_paragraph,
    clippy::arbitrary_source_item_ordering,
    clippy::missing_panics_doc
)]
#![allow(
    clippy::unnecessary_wraps,
    clippy::use_self,
    mismatched_lifetime_syntaxes,
    dead_code
)]
#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]
pub mod byte_ranges;
pub mod const_checked;
pub mod jeb85;
pub mod model;
pub mod nodes;
pub mod z85;
pub use {
    crate::{
        byte_ranges::*,
        common::*,
        const_checked::*,
        value::*,
    },
    jeb_common as common,
    jeb_stream as streams,
    jeb_value as value,
};
