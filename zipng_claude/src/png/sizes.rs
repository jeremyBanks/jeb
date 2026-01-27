//! Constants related to the PNG file format, which may be relevant for both
//! [reading][super::reading] and [writing][super::writing].

pub const PNG_CHUNK_PREFIX_SIZE: usize = 8;
pub const PNG_CHUNK_SUFFIX_SIZE: usize = 4;
pub const PNG_CHUNK_WRAPPER_SIZE: usize = PNG_CHUNK_PREFIX_SIZE + PNG_CHUNK_SUFFIX_SIZE;

/// Size in bytes of the PNG signature and IHDR chunk together.
pub const PNG_HEADER_SIZE: usize = 33;
