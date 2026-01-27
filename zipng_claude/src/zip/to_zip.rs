use {
    crate::{default, Zip},
    std::borrow::Cow::{self, Owned},
};

/// A [`Zip`] or input that can be converted to one.
pub trait ToZip {
    fn to_zip(&self) -> Cow<Zip>;
}

impl ToZip for Zip {
    fn to_zip(&self) -> Cow<Zip> {
        Cow::Borrowed(self)
    }
}

impl ToZip for fn(&mut Zip) {
    /// Create a [`Zip`] from a function that mutates a [`Zip::default`].
    fn to_zip(&self) -> Cow<Zip> {
        let mut zip = default();
        self(&mut zip);
        Owned(zip)
    }
}

impl ToZip for fn(Zip) -> Zip {
    /// Create a [`Zip`] from a function that mutates a [`Zip::default`].
    fn to_zip(&self) -> Cow<Zip> {
        Owned(self(default()))
    }
}
