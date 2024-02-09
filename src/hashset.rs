use crate::HashMap;
use core::{borrow::Borrow, hash::Hash};
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

impl<T, S> Default for HashSet<T, S>
where
    S: Default,
{
    fn default() -> Self {
        Self { inner: HashMap::<_, _, S>::default() }
    }
}
