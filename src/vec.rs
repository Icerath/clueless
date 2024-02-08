use core::{
    fmt, mem,
    ops::{Deref, DerefMut},
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
        self.len -= 1;
        Some(unsafe { self.buf.read(self.len) })
    }
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
    pub fn insert(&mut self, index: usize, val: T) {
        assert!(self.try_insert(index, val).is_ok(), "index was {index} when len was {}", self.len);
    }
    pub fn remove(&mut self, index: usize) -> T {
        match self.try_remove(index) {
            Ok(val) => val,
            Err(IndexNotFound) => panic!("index was {index} when len was {}", self.len),
        }
    }
    pub fn try_swap_remove(&mut self, index: usize) -> Result<T, IndexNotFound> {
        if self.len == 0 {
            return Err(IndexNotFound);
        }
        let len = self.len;
        self.swap(index, len - 1);
        self.pop().ok_or(IndexNotFound)
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        match self.try_swap_remove(index) {
            Ok(val) => val,
            Err(IndexNotFound) => panic!("index was {index} when len was {}", self.len),
        }
    }
    /// Makes space for at least additional MORE elem while keeping exponential growth.
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
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
impl<T> fmt::Debug for Vec<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
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