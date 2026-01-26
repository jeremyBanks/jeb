use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::hash::Hash;

use static_assertions::assert_impl_all;
use thiserror::Error;

use crate::generic::default;

pub trait Primitive: Copy {}
impl<T> Primitive for T where T: Copy {}

assert_impl_all!(u8: Primitive);
assert_impl_all!(char: Primitive);
assert_impl_all!((u32, u32): Primitive);
assert_impl_all!(crate::Blip<str>: Primitive);
assert_impl_all!(InlineVec<InlineVec<f32, 2>, 4>: Primitive);

/// Our own limited implementation of `arrayvec`, but `Copy`.
pub struct InlineVec<T: Primitive, const CAPACITY: usize> {
    length: usize,
    buffer: [T; CAPACITY],
}

impl<T: Primitive, const CAPACITY: usize> InlineVec<T, CAPACITY> {
    pub fn new() -> Self
    where T: Default {
        default()
    }

    pub fn try_from_slice(slice: &[T]) -> Result<Self, CapacityExceeded>
    where T: Default {
        slice.try_into()
    }

    pub fn try_from_vec(slice: Vec<T>) -> Result<Self, CapacityExceeded>
    where T: Default {
        slice.as_slice().try_into()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.length == CAPACITY
    }

    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            Err(value)
        } else {
            self.buffer[self.length] = value;
            self.length += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;
            Some(self.buffer[self.length])
        }
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn iter(&self) -> <Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T: Primitive, const CAPACITY: usize> Default for InlineVec<T, CAPACITY>
where T: Default
{
    fn default() -> Self {
        Self {
            length: default(),
            buffer: [default(); CAPACITY],
        }
    }
}

impl<T: Primitive, const CAPACITY: usize> AsRef<[T]> for InlineVec<T, CAPACITY> {
    fn as_ref(&self) -> &[T] {
        &self.buffer[0..self.length]
    }
}

impl<T: Primitive, const CAPACITY: usize> AsMut<[T]> for InlineVec<T, CAPACITY> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.buffer[0..self.length]
    }
}

impl<T: Primitive, const CAPACITY: usize> Borrow<[T]> for InlineVec<T, CAPACITY> {
    fn borrow(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T: Primitive, const CAPACITY: usize> BorrowMut<[T]> for InlineVec<T, CAPACITY> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T: Primitive, const CAPACITY: usize> Hash for InlineVec<T, CAPACITY>
where T: Hash
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
impl<T: Primitive, const CAPACITY: usize> Eq for InlineVec<T, CAPACITY> where T: Eq {}
impl<T: Primitive, const CAPACITY: usize> PartialEq for InlineVec<T, CAPACITY>
where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}
impl<T: Primitive, const CAPACITY: usize> Ord for InlineVec<T, CAPACITY>
where T: Ord
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}
impl<T: Primitive, const CAPACITY: usize> PartialOrd for InlineVec<T, CAPACITY>
where T: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}
impl<T: Primitive, const CAPACITY: usize> Debug for InlineVec<T, CAPACITY>
where T: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl<T: Primitive, const CAPACITY: usize> Clone for InlineVec<T, CAPACITY> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Primitive, const CAPACITY: usize> Copy for InlineVec<T, CAPACITY> {}

impl<T: Primitive, const CAPACITY: usize> From<heapless::Vec<T, CAPACITY>>
    for InlineVec<T, CAPACITY>
where T: Default
{
    fn from(value: heapless::Vec<T, CAPACITY>) -> Self {
        value.as_slice().try_into().unwrap()
    }
}

#[derive(Debug, Error)]
#[error("Array capacity exceeded")]
pub struct CapacityExceeded;

impl<T: Primitive, const CAPACITY: usize> TryFrom<&[T]> for InlineVec<T, CAPACITY>
where T: Default
{
    type Error = CapacityExceeded;

    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        let length = value.len();
        if length <= CAPACITY {
            let mut buffer = [default(); CAPACITY];
            buffer[..length].copy_from_slice(value);
            Ok(Self { buffer, length })
        } else {
            Err(CapacityExceeded)
        }
    }
}

impl<T: Primitive, const CAPACITY: usize> From<[T; CAPACITY]> for InlineVec<T, CAPACITY> {
    fn from(value: [T; CAPACITY]) -> Self {
        Self {
            length: CAPACITY,
            buffer: value,
        }
    }
}

impl<T: Primitive, const CAPACITY: usize> IntoIterator for InlineVec<T, CAPACITY> {
    type Item = T;
    type IntoIter = std::iter::Take<std::array::IntoIter<T, CAPACITY>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter().take(self.length)
    }
}
