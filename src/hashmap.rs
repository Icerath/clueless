use crate::Vec;

use core::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    mem,
};
use std::hash::RandomState;

pub struct HashMap<K, V, H = RandomState> {
    buckets: Box<[Bucket<K, V>]>,
    hasher: H,
}

type Bucket<K, V> = Vec<(K, V)>;
impl<K, V, H> HashMap<K, V, H>
where
    H: Default,
{
    #[must_use]
    pub fn new() -> Self {
        Self { buckets: Box::from([]), hasher: H::default() }
    }
}

impl<K, V> HashMap<K, V> {
    const MAX_BUCKET_LEN: usize = 8;
    const START_CAPACITY: usize = 8;
    #[must_use]
    pub fn len(&self) -> usize {
        self.buckets.iter().map(Bucket::len).sum()
    }
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.buckets.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, val: V) {
        if self.buckets.is_empty() {
            self.grow();
        }
        let bucket = self.get_bucket(&key);
        if self.buckets[bucket].len() == Self::MAX_BUCKET_LEN {
            self.grow();
        }
        self.buckets[bucket].push((key, val));
    }
    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.get_bucket(key);
        self.buckets[bucket].iter().find_map(|(k, v)| (k.borrow() == key).then_some(v))
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.remove_entry(key).map(|(_k, v)| v)
    }
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.get_bucket(key);
        let index = self.buckets[bucket].iter().position(|(k, _v)| k.borrow() == key)?;
        Some(self.buckets[bucket].remove(index))
    }
    #[allow(clippy::cast_possible_truncation)]
    fn get_bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = self.hasher.hash_one(key);
        let bucket = hash % self.buckets.len() as u64;
        bucket as usize
    }
    fn grow(&mut self) {
        if self.buckets.is_empty() {
            return self.buckets =
                std::iter::repeat_with(Vec::new).take(Self::START_CAPACITY).collect();
        }
        let new_buckets = std::iter::repeat_with(Vec::new).take(self.buckets.len() * 2).collect();
        let old_buckets = mem::replace(&mut self.buckets, new_buckets);
        for (key, val) in Vec::from(old_buckets).into_iter().flatten() {
            let bucket = self.get_bucket(&key);
            self.buckets[bucket].push((key, val));
        }
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
pub fn test_basics() {
    let mut map = HashMap::new();

    assert_eq!(map.capacity(), 0);
    assert!(map.is_empty());

    map.insert("foo", 1);
    map.insert("bar", 2);
    map.insert("baz", 3);

    assert_eq!(map.len(), 3);

    assert_eq!(map.get("foo"), Some(&1));
    assert_eq!(map.get("bar"), Some(&2));
    assert_eq!(map.get("baz"), Some(&3));

    assert_eq!(map.remove("foo"), Some(1));
    assert_eq!(map.remove("bar"), Some(2));
    assert_eq!(map.remove("baz"), Some(3));

    assert!(map.is_empty());
}

#[test]
pub fn test_growth() {
    let mut map = HashMap::new();
    for i in 0..100 {
        map.insert(i, i);
    }
    assert_ne!(map.capacity(), HashMap::<(), ()>::START_CAPACITY);
}
