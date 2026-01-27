use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitCode;
use std::process::Termination;

use static_assertions::assert_impl_all;

pub fn default<T>() -> T
where T: Default {
    T::default()
}

/// equivalent to [`core::mem::drop`]
pub fn noop_move<T>(_x: T) {}
/// equivalent to [`core::convert::identity`]
pub fn noop_move_move<T>(x: T) -> T {
    x
}
pub fn noop_ref<T>(_x: &T) {}
pub fn noop_ref_ref<T>(x: &T) -> &T {
    x
}
pub fn noop_mut<T>(_x: &mut T) {}
pub fn noop_mut_mut<T>(x: &mut T) -> &mut T {
    x
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
/// An uninhabited [`!`]-like "never" type, with trait implementations as needed
/// for convenience within this crate's types.
pub enum never {}

assert_impl_all!(never: Send, Sync);
assert_impl_all!(panic: Send, Sync);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
/// An uninhabited [`!`]-like "never" type that provides a panicking
/// implementation of `From` for any `Display + Debug` error type,
/// with trait implementations as needed for convenience within this crate's.
pub enum panic {}

impl<Err> From<Err> for panic
where Err: Display + Debug
{
    #[track_caller]
    fn from(error: Err) -> Self {
        panic!("{error}")
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

impl From<never> for panic {
    fn from(_: never) -> panic {
        unreachable!()
    }
}

impl Termination for panic {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}

impl Termination for never {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}

use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

/// This is a convenience wrapper for `PhantomData<fn(T) -> T>`, which
/// seems to be the right way to defined a `PhantomData` without affecting
/// either the borrow checker (lifetimes) or the drop checker (ownership,
/// borrowing).
pub(crate) struct PhantomType<T: ?Sized>(PhantomData<fn(T) -> T>);

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
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T: ?Sized> Hash for PhantomType<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {}
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
