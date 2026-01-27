use {
    crate::{default, Zip},
    std::{cmp::Ordering, fmt::Debug},
};

/// A reference to an entry in a [`Zip`] archive.
pub struct ZipEntry<'name, 'body> {
    pub name: &'name [u8],
    pub body: &'body [u8],
}

/// Predicate function used for selecting entries in a [`Zip`] archive.
pub type ZipEntryPredicate = fn(&ZipEntry) -> bool;

/// Comparison function used for ordering entries in a [`Zip`] archive.
pub type ZipEntryComparison = fn(&ZipEntry, &ZipEntry) -> Ordering;

/// Sort file by name, first based on the number of slash `/` path separator
/// characters, then lexicographically.
pub static SORT_BY_NAME: ZipEntryComparison = |a, b| {
    (
        a.name.iter().filter(|&&c| c == b'/').collect::<Vec<_>>(),
        a.name,
    )
        .cmp(&(
            b.name.iter().filter(|&&c| c == b'/').collect::<Vec<_>>(),
            b.name,
        ))
};

/// Sorts files lexicographically based on their content, then applies
/// [`SORT_BY_NAME`].
pub static SORT_BY_BODY: ZipEntryComparison = |a, b| {
    (a.body, a.name)
        .cmp(&(b.body, b.name))
        .then_with(|| SORT_BY_NAME(a, b))
};

/// Sorts files by size class, then applies [`SORT_BY_BODY`].
pub static SORT_BY_SIZE: ZipEntryComparison = |a, b| {
    fn size_class(n: usize) -> usize {
        match n {
            0x00..=0b00 => 0x00,
            0x01..=0x40 => 0x40,
            0x41..=0x100 => 0x100,
            0x101..=0x400 => 0x400,
            n => n.next_power_of_two(),
        }
    }

    let a_size = size_class(a.body.len());

    let b_size = size_class(b.body.len());
    a_size.cmp(&b_size).then_with(|| SORT_BY_BODY(a, b))
};

/// Pins the `mimetype` magic header file used by EPUB and OpenDocument.
pub static PIN_MIMETYPE: ZipEntryPredicate =
    |file| file.name == b"mimetype" && file.body.is_ascii() && file.body.len() <= 0xFF;

/// Configuration determining how the [`Zip`] archive is serialized.
#[derive(Clone)]
#[non_exhaustive]
pub struct ZipConfiguration {
    /// Whether all body data must be stored contiguously.
    pub body_contiguous: bool,
    /// Align and pad all body data into blocks of this many bytes.
    pub body_alignment: usize,
    /// Comparison function used to determine file body ordering.
    pub sort_body_by: Option<ZipEntryComparison>,
    /// Comparison function used to determine file metadata ordering.
    pub sort_meta_by: Option<ZipEntryComparison>,
    /// Predicate function used to determine files that should be pinned to the
    /// top of the archive without alignment or padding.
    pub pin_header_with: Option<ZipEntryPredicate>,
}

impl ZipConfiguration {
    pub fn new(f: fn(&mut Self)) -> Self {
        let mut zip_configuration = default();
        f(&mut zip_configuration);

        zip_configuration
    }
}

impl Debug for ZipConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZipConfiguration")
            .field("body_contiguous", &self.body_contiguous)
            .field("body_alignment", &self.body_alignment)
            .field(
                "sort_body_by",
                self.sort_body_by.map(|_| &Some(())).unwrap_or(&None),
            )
            .field(
                "sort_meta_by",
                self.sort_meta_by.map(|_| &Some(())).unwrap_or(&None),
            )
            .finish()
    }
}

impl Default for ZipConfiguration {
    fn default() -> Self {
        Self {
            body_contiguous: true,
            body_alignment: 1024,
            sort_body_by: SORT_BY_BODY.into(),
            sort_meta_by: SORT_BY_NAME.into(),
            pin_header_with: PIN_MIMETYPE.into(),
        }
    }
}

impl Zip {
    /// Configure this archive to avoid including anything extraneous.
    pub fn minimal(&mut self) {
        todo!()
    }

    /// Configures this archive to store file contents contiguously, aligned
    /// and padded to to 1024-byte blocks.
    pub fn blocks(&mut self) {
        todo!()
    }

    /// Configures this archive to create a polyglot PNG/ZIP file which can also
    /// be interpreted as a PNG image image file displaying the content of the
    /// first PNG image file that the archive contains.
    pub fn covered(&mut self) {
        todo!()
    }

    /// Configures this archive to create a polyglot PNG/ZIP file which can also
    /// be interpreted as a PNG image with all of the file contents displayed in
    /// the image data.
    pub fn uncovered(&mut self) {
        todo!()
    }
}
