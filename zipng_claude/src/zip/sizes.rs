//! Constants related to the PNG file format, which may be relevant for both
//! [reading][super::reading] and [writing][super::writing].

/// The size of an individual file header for a zip file, assuming all
/// variable-length fields are empty (i.e. zero-length filename, comment, and
/// extra data).
pub const ZIP_FILE_HEADER_EMPTY_SIZE: usize = 0;
