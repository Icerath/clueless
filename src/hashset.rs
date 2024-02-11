use crate::HashMap;
use core::{borrow::Borrow, fmt, hash::Hash};
use std::hash::{BuildHasher, RandomState};

pub struct HashSet<T, S = RandomState> {
    inner: HashMap<T, (), S>,
}

impl<T> HashSet<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }
}
impl<T, S> HashSet<T, S> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.into_iter()
    }
}
impl<T, S> HashSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    pub fn insert(&mut self, val: T) -> Option<T> {
        self.inner.insert(val, ()).map(|entry| entry.0)
    }
    pub fn remove<Q>(&mut self, val: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.remove_entry(val).map(|entry| entry.0)
    }
}

impl<T, S> FromIterator<T> for HashSet<T, S>
where
    T: Hash + Eq,
    S: Default + BuildHasher,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self { inner: iter.into_iter().map(|key| (key, ())).collect() }
    }
}

impl<T, S> IntoIterator for HashSet<T, S> {
    type Item = T;
    type IntoIter = impl Iterator<Item = T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().map(|entry| entry.0)
    }
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S> {
    type Item = &'a T;
    type IntoIter = impl Iterator<Item = &'a T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().map(|entry| entry.0)
    }
}

impl<T, S> Default for HashSet<T, S>
where
    S: Default,
{
    fn default() -> Self {
        Self { inner: HashMap::<_, _, S>::default() }
    }
}

impl<T, S> fmt::Debug for HashSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}
