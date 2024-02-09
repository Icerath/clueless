use crate::Vec;

use core::{
    borrow::Borrow,
    fmt,
    hash::{BuildHasher, Hash},
    mem,
};
use std::hash::RandomState;

pub struct HashMap<K, V, H = RandomState> {
    buckets: Box<[Bucket<K, V>]>,
    hasher: H,
}

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
    const MAX_BUCKET_LEN: usize = 6;
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
    #[must_use]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    #[must_use]
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
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
        self.buckets[bucket].push(key, val);
    }
    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.get_bucket(key);
        self.buckets[bucket].get(key)
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
        self.buckets[bucket].remove(key)
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
                std::iter::repeat_with(Bucket::new).take(Self::START_CAPACITY).collect();
        }
        let new_buckets =
            std::iter::repeat_with(Bucket::new).take(self.buckets.len() * 2).collect();
        let old_buckets = mem::replace(&mut self.buckets, new_buckets);
        for node in Vec::from(old_buckets).into_iter().flatten() {
            let bucket = self.get_bucket(&node.key);
            self.buckets[bucket].push_node(node);
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = impl Iterator<Item = (K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self.buckets)
            .into_iter()
            .flat_map(|bucket| bucket.into_iter().map(|node| (node.key, node.val)))
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = impl Iterator<Item = (&'a K, &'a V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.buckets.iter().flatten()
    }
}

impl<'a, K, V> IntoIterator for &'a mut HashMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = impl Iterator<Item = (&'a K, &'a mut V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.buckets.iter_mut().flatten()
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> fmt::Debug for HashMap<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

struct Bucket<K, V> {
    head: Option<Box<Node<K, V>>>,
}
struct Node<K, V> {
    next: Option<Box<Node<K, V>>>,
    key: K,
    val: V,
}

impl<K, V> Bucket<K, V> {
    const fn new() -> Self {
        Self { head: None }
    }
    const fn len(&self) -> usize {
        let mut head = &self.head;
        let mut len = 0;
        while let Some(current) = head {
            len += 1;
            head = &current.next;
        }
        len
    }
    fn push(&mut self, key: K, val: V) {
        self.push_node(Box::new(Node { next: None, key, val }));
    }
    fn push_node(&mut self, val: Box<Node<K, V>>) {
        let mut head = &mut self.head;
        while let Some(current) = head {
            head = &mut current.next;
        }
        assert!(head.is_none());
        head.replace(val);
    }
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let mut head = &self.head;
        while let Some(current) = head {
            if current.key.borrow() == key {
                return Some(&current.val);
            }
            head = &current.next;
        }
        None
    }
    fn remove<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let mut current = &mut self.head;
        loop {
            match current {
                None => return None,
                Some(node) if node.key.borrow() == key => {
                    let mut node = current.take().unwrap();
                    *current = node.next.take();
                    return Some((node.key, node.val));
                }
                Some(node) => {
                    current = &mut node.next;
                }
            }
        }
    }
}

impl<K, V> IntoIterator for Bucket<K, V> {
    type IntoIter = impl Iterator<Item = Box<Node<K, V>>>;
    type Item = Box<Node<K, V>>;
    fn into_iter(self) -> Self::IntoIter {
        let mut self_current = self.head;
        std::iter::from_fn(move || {
            let mut current = mem::take(&mut self_current)?;
            self_current = current.next;
            current.next = None;
            Some(current)
        })
    }
}

impl<'a, K, V> IntoIterator for &'a Bucket<K, V> {
    type IntoIter = impl Iterator<Item = (&'a K, &'a V)>;
    type Item = (&'a K, &'a V);
    fn into_iter(self) -> Self::IntoIter {
        let mut self_current = self.head.as_deref();
        std::iter::from_fn(move || {
            let current = self_current?;
            self_current = current.next.as_deref();
            Some((&current.key, &current.val))
        })
    }
}

impl<'a, K, V> IntoIterator for &'a mut Bucket<K, V> {
    type IntoIter = impl Iterator<Item = (&'a K, &'a mut V)>;
    type Item = (&'a K, &'a mut V);
    fn into_iter(self) -> Self::IntoIter {
        let mut self_current = self.head.as_deref_mut();
        std::iter::from_fn(move || {
            let current = mem::take(&mut self_current)?;
            self_current = current.next.as_deref_mut();
            Some((&current.key, &mut current.val))
        })
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
        map.insert(i, i * 2);
    }
    assert_ne!(map.capacity(), HashMap::<(), ()>::START_CAPACITY);
}
