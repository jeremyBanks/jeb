use {
    static_assertions::assert_impl_all,
    std::{
        fmt::{Debug, Display},
        process::{ExitCode, Termination},
    },
};

pub fn default<T>() -> T
where T: Default {
    T::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[doc(hidden)]
/// An uninhabited [`!`]-like "never" type, with trait implementations as needed
/// for convenience within this crate's types.
pub enum never {}
assert_impl_all!(never: Send, Sync);

impl Display for never {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}

impl Default for never {
    fn default() -> Self {
        unreachable!()
    }
}

impl From<panic> for never {
    fn from(_: panic) -> never {
        unreachable!()
    }
}

impl Termination for never {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
/// An uninhabited [`!`]-like "never" type that provides a panicking
/// implementation of `From` for any `Display + Debug` error type,
/// with trait implementations as needed for convenience within this crate's.
#[doc(hidden)]
pub enum panic {}
assert_impl_all!(panic: Send, Sync);

impl<Err> From<Err> for panic
where Err: Display + Debug
{
    #[track_caller]
    fn from(error: Err) -> Self {
        panic!("{error}")
    }
}

impl Termination for panic {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}

use std::{cmp::Ordering, fmt, hash::Hash, marker::PhantomData};

/// This is a convenience wrapper for `PhantomData<fn(T) -> T>`, which
/// seems to be the right way to defined a `PhantomData` without affecting
/// either the borrow checker (lifetimes) or the drop checker (ownership,
/// borrowing).
pub struct PhantomType<T: ?Sized>(PhantomData<fn(T) -> T>);

impl<T: ?Sized> Copy for PhantomType<T> {}

assert_impl_all!(
  PhantomType<(*const u8, dyn Debug)>:
    Copy,
    Send,
    Sync,
    Default,
    Sized,
    Eq,
    Ord,
    Hash,
    From<()>,
    Into<()>,
);

impl<T: ?Sized> Clone for PhantomType<T> {
    fn clone(&self) -> Self {
        PhantomType(PhantomData)
    }
}

impl<T: ?Sized> PartialEq for PhantomType<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: ?Sized> Eq for PhantomType<T> {}

impl<T: ?Sized> PartialOrd for PhantomType<T> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<T: ?Sized> Ord for PhantomType<T> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T: ?Sized> Hash for PhantomType<T> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl<T: ?Sized> PhantomType<T> {
    pub fn new() -> Self {
        PhantomType(PhantomData)
    }
}

impl<T: ?Sized> Debug for PhantomType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("👻")
    }
}

impl<T: ?Sized> Default for PhantomType<T> {
    fn default() -> Self {
        PhantomType(PhantomData)
    }
}

impl<T: ?Sized> From<()> for PhantomType<T> {
    fn from(_: ()) -> Self {
        PhantomType(PhantomData)
    }
}

impl<T: ?Sized> From<PhantomType<T>> for () {
    fn from(_: PhantomType<T>) -> Self {}
}
