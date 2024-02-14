#![allow(unsafe_code)]

use alloc::boxed::Box;
use core::{
    fmt,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(clippy::module_name_repetitions)]
pub use crate::raw_vec::RawVec;

pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexNotFound;

impl<T> Vec<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: RawVec::new(), len: 0 }
    }

    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        let mut ret = Self::new();
        ret.reserve(cap);
        ret
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }
        unsafe { self.buf.write(self.len, val) };
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.pop_unchecked() })
    }

    unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());
        self.len -= 1;
        unsafe { self.buf.read(self.len) }
    }

    /// # Errors
    /// Will return an Err when `index > len`
    pub fn try_insert(&mut self, index: usize, val: T) -> Result<(), IndexNotFound> {
        if index > self.len {
            return Err(IndexNotFound);
        }
        if self.len == self.cap() {
            self.buf.grow();
        }
        unsafe {
            self.buf.shift(index, index + 1, self.len - index);
            self.buf.write(index, val);
        }
        self.len += 1;
        Ok(())
    }

    /// # Errors
    /// Will return an Err when `index >= len`
    pub fn try_remove(&mut self, index: usize) -> Result<T, IndexNotFound> {
        if index >= self.len {
            return Err(IndexNotFound);
        }
        self.len -= 1;
        unsafe {
            let result = self.buf.read(index);
            self.buf.shift(index + 1, index, self.len - index);
            Ok(result)
        }
    }

    /// # Errors
    /// Will return an Err when `index >= len`
    pub fn try_swap_remove(&mut self, index: usize) -> Result<T, IndexNotFound> {
        if index >= self.len {
            return Err(IndexNotFound);
        }
        let len = self.len;
        self.swap(index, len - 1);
        // Safety:
        // the first check guarantees len cannot be zero
        Ok(unsafe { self.pop_unchecked() })
    }

    /// # Panics
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, val: T) {
        self.try_insert(index, val)
            .unwrap_or_else(|_| panic!("index was {index} when len was {}", self.len));
    }

    /// # Panics
    /// Panics if `index >= len`.
    pub fn remove(&mut self, index: usize) -> T {
        self.try_remove(index)
            .unwrap_or_else(|_| panic!("index was {index} when len was {}", self.len))
    }

    /// # Panics
    /// Panics if `index >= len`.
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.try_swap_remove(index)
            .unwrap_or_else(|_| panic!("index was {index} when len was {}", self.len))
    }

    /// Makes space for at least additional MORE elem while keeping exponential
    /// growth.
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.buf.resize(self.len());
    }

    #[must_use]
    pub fn into_boxed_slice(mut self) -> Box<[T]> {
        self.shrink_to_fit();
        let mut vec = ManuallyDrop::new(self);
        unsafe { Box::from_raw(vec.as_slice_mut()) }
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for val in iter {
            self.push(val);
        }
    }
}

impl<T> Vec<T> {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn cap(&self) -> usize {
        self.buf.cap
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> Clone for Vec<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T> core::ops::Index<usize> for Vec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T> core::ops::IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}

impl<T> fmt::Debug for Vec<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> From<Box<[T]>> for Vec<T> {
    fn from(value: Box<[T]>) -> Self {
        let cap = value.len();
        let ptr = NonNull::from(Box::leak(value)).cast();

        Self { buf: RawVec { ptr, cap }, len: cap }
    }
}

impl<T> PartialEq for Vec<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other)
    }
}

impl<T> Eq for Vec<T> where T: Eq {}

impl<T> IntoIterator for Vec<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(mut self) -> Self::IntoIter {
        let buf = mem::take(&mut self.buf);
        let len = self.len;
        mem::forget(self);

        IntoIter { buf, current: 0, end: len }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type IntoIter = core::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type IntoIter = core::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<T> {
    buf: RawVec<T>,
    current: usize,
    end: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let val = unsafe { self.buf.read(self.current) };
        self.current += 1;
        Some(val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let val = unsafe { self.buf.read(self.end) };
        self.end -= 1;
        Some(val)
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.end - self.current
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Self::default();
        vec.extend(iter);
        vec
    }
}

#[test]
fn test_iters() {
    use alloc::string::String;
    let mut items: Vec<_> = (0..10).collect();
    let _ = items.clone().into_iter();

    assert!(items.iter().copied().eq(0..10));
    assert!(items.iter_mut().map(|x| *x).eq(0..10));
    assert!(items.into_iter().eq(0..10));

    let mut strings = ["lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);
    let mut items: Vec<String> = strings.iter().cloned().collect();
    let _ = items.clone().into_iter();

    assert!(items.iter().eq(&strings));
    assert!(items.iter_mut().eq(&mut strings));
    assert!(items.into_iter().eq(strings));
}

#[test]
fn test_insert_remove() {
    let mut items = Vec::new();

    for i in 0..10 {
        items.insert(0, i);
    }

    for i in (0..10).rev() {
        assert_eq!(items.remove(0), i);
    }
}

#[test]
fn test_boxed_slice() {
    let pre = (0..10).collect::<Vec<_>>();
    let post = pre.clone().into_boxed_slice().into();
    assert_eq!(pre, post);
}
