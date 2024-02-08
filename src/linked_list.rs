use crate::vec::{self, Vec};
use core::fmt;

pub mod small {
    #[allow(private_interfaces)]
    pub type LinkedList<T> = super::LinkedList<T, u32>;
}
pub mod large {
    #[allow(private_interfaces)]
    pub type LinkedList<T> = super::LinkedList<T, usize>;
}

pub trait Index: Eq + Copy + private::Sealed {
    const NIL: Self;
    fn usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
}

mod private {
    pub trait Sealed {}

    impl Sealed for usize {}
    impl Sealed for u32 {}
}

impl Index for u32 {
    const NIL: Self = Self::MAX;
    #[inline]
    fn usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn from_usize(val: usize) -> Self {
        val.try_into().expect("Linked List grew too large")
    }
}

impl Index for usize {
    const NIL: Self = Self::MAX;
    fn usize(self) -> usize {
        self
    }
    fn from_usize(val: usize) -> Self {
        val
    }
}

#[derive(Clone)]
pub struct LinkedList<T, Idx = usize> {
    buf: Vec<Node<T, Idx>>,
    head: Idx,
    tail: Idx,
}

impl<T, Idx> LinkedList<T, Idx>
where
    Idx: Index,
{
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: Vec::new(), head: Idx::NIL, tail: Idx::NIL }
    }
    pub fn push_back(&mut self, val: T) {
        let ptr = self.push_buf(Node { val, prev: self.tail, next: Idx::NIL });
        if self.head == Idx::NIL {
            self.head = ptr;
        }
        if self.tail != Idx::NIL {
            self.buf[self.tail.usize()].next = ptr;
        }
        self.tail = ptr;
    }
    pub fn push_front(&mut self, val: T) {
        let ptr = self.push_buf(Node { val, next: self.head, prev: Idx::NIL });
        if self.tail == Idx::NIL {
            self.tail = ptr;
        }
        if self.head != Idx::NIL {
            self.buf[self.head.usize()].prev = ptr;
        }
        self.head = ptr;
    }
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let node = self.remove_node(self.tail);
        self.tail = node.prev;
        if self.tail != Idx::NIL {
            self.buf[self.tail.usize()].next = Idx::NIL;
        }
        Some(node.val)
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let node = self.remove_node(self.head);
        self.head = node.next;
        if self.head != Idx::NIL {
            self.buf[self.head.usize()].prev = Idx::NIL;
        }
        Some(node.val)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T, Idx> {
        self.into_iter()
    }
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, Idx> {
        self.into_iter()
    }
    fn remove_node(&mut self, ptr: Idx) -> Node<T, Idx> {
        let end = &self.buf[self.len() - 1];
        let (end_prev, end_next) = (end.prev, end.next);

        if let Some(prev) = self.buf.get_mut(end_prev.usize()) {
            prev.next = ptr;
        }
        if let Some(next) = self.buf.get_mut(end_next.usize()) {
            next.prev = ptr;
        }
        let node = self.buf.swap_remove(ptr.usize());

        if self.head.usize() == self.len() {
            self.head = ptr;
        }
        if self.tail.usize() == self.len() {
            self.tail = ptr;
        }
        node
    }
    fn push_buf(&mut self, node: Node<T, Idx>) -> Idx {
        let ptr = self.buf.len();
        self.buf.push(node);
        Idx::from_usize(ptr)
    }
}

impl<T, Idx> Default for LinkedList<T, Idx>
where
    Idx: Index,
{
    fn default() -> Self {
        Self::new()
    }
}

// IntoIter
pub struct IntoIter<T, Idx> {
    list: LinkedList<T, Idx>,
}

impl<T, Idx> Iterator for IntoIter<T, Idx>
where
    Idx: Index,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T, Idx> DoubleEndedIterator for IntoIter<T, Idx>
where
    Idx: Index,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T, Idx> ExactSizeIterator for IntoIter<T, Idx> where Idx: Index {}

impl<T, Idx> IntoIterator for LinkedList<T, Idx>
where
    Idx: Index,
{
    type IntoIter = IntoIter<T, Idx>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}
// Iter
pub struct Iter<'a, T, Idx> {
    list: &'a LinkedList<T, Idx>,
    head: Idx,
    tail: Idx,
    len: usize,
}

impl<'a, T, Idx> Iterator for Iter<'a, T, Idx>
where
    Idx: Index,
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let val = &self.list.buf[self.head.usize()].val;
        self.head = self.list.buf[self.head.usize()].next;
        Some(val)
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T, Idx> DoubleEndedIterator for Iter<'_, T, Idx>
where
    Idx: Index,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let val = &self.list.buf[self.tail.usize()].val;
        self.tail = self.list.buf[self.tail.usize()].prev;
        Some(val)
    }
}

impl<T, Idx> ExactSizeIterator for Iter<'_, T, Idx> where Idx: Index {}

impl<'a, T, Idx> IntoIterator for &'a LinkedList<T, Idx>
where
    Idx: Index,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, Idx>;
    fn into_iter(self) -> Self::IntoIter {
        Iter { list: self, head: self.head, tail: self.tail, len: self.len() }
    }
}

// IterMut
pub struct IterMut<'a, T, Idx> {
    list: &'a mut LinkedList<T, Idx>,
    head: Idx,
    tail: Idx,
    len: usize,
}

impl<'a, T, Idx> Iterator for IterMut<'a, T, Idx>
where
    Idx: Index,
{
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let head = self.head;
        self.head = self.list.buf[self.head.usize()].next;
        let val = &mut self.list.buf[head.usize()].val;

        // This makes me sad
        Some(unsafe { std::mem::transmute(val) })
    }
}

impl<'a, T, Idx> DoubleEndedIterator for IterMut<'a, T, Idx>
where
    Idx: Index,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let tail = self.tail;
        self.tail = self.list.buf[self.tail.usize()].prev;
        let val = &mut self.list.buf[tail.usize()].val;

        // This makes me sad
        Some(unsafe { std::mem::transmute(val) })
    }
}

impl<'a, T, Idx> IntoIterator for &'a mut LinkedList<T, Idx>
where
    Idx: Index,
{
    type IntoIter = IterMut<'a, T, Idx>;
    type Item = &'a mut T;
    fn into_iter(self) -> Self::IntoIter {
        IterMut { head: self.head, tail: self.tail, len: self.len(), list: self }
    }
}

// FromIter
impl<T, Idx> FromIterator<T> for LinkedList<T, Idx>
where
    Idx: Index,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut list = Self::default();
        list.buf.reserve(iter.size_hint().0);
        for i in iter {
            list.push_back(i);
        }
        list
    }
}

// IntoIter Unordered
type IntoIterUnordered<T, F, Idx> = core::iter::Map<vec::IntoIter<Node<T, Idx>>, F>;

impl<T, Idx> LinkedList<T, Idx> {
    pub fn into_iter_unordered(self) -> IntoIterUnordered<T, impl FnMut(Node<T, Idx>) -> T, Idx> {
        self.buf.into_iter().map(|node| node.val)
    }
}
// Iter Unordered
type IterUnordered<'a, T, F, Idx> = core::iter::Map<core::slice::Iter<'a, Node<T, Idx>>, F>;

impl<T, Idx> LinkedList<T, Idx>
where
    Idx: Index,
{
    pub fn iter_unordered<'a>(
        &'a self,
    ) -> IterUnordered<'_, T, impl FnMut(&'a Node<T, Idx>) -> &'a T, Idx> {
        self.buf.iter().map(|node| &node.val)
    }
}

// IterMut Unordered
type IterMutUnordered<'a, T, F, Idx> = core::iter::Map<std::slice::IterMut<'a, Node<T, Idx>>, F>;

impl<T, Idx> LinkedList<T, Idx>
where
    Idx: Index,
{
    pub fn iter_mut_unordered<'a>(
        &'a mut self,
    ) -> IterMutUnordered<'_, T, impl FnMut(&'a mut Node<T, Idx>) -> &'a mut T, Idx> {
        self.buf.iter_mut().map(|node| &mut node.val)
    }
}

impl<T, Idx> fmt::Debug for LinkedList<T, Idx>
where
    T: fmt::Debug,
    Idx: Index,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

#[derive(Clone)]
pub struct Node<T, Idx> {
    val: T,
    next: Idx,
    prev: Idx,
}

#[test]
fn test_basics() {
    let mut list = small::LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    assert_eq!(list.pop_back(), Some(3));
    assert_eq!(list.pop_back(), Some(2));
    assert_eq!(list.pop_back(), Some(1));

    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), Some(3));

    list.push_front(1);
    list.push_front(2);
    list.push_front(3);
    assert_eq!(list.pop_front(), Some(3));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), Some(1));

    list.push_front(1);
    list.push_front(2);
    list.push_front(3);
    assert_eq!(list.pop_back(), Some(1));
    assert_eq!(list.pop_back(), Some(2));
    assert_eq!(list.pop_back(), Some(3));
}

#[allow(clippy::cognitive_complexity)]
#[test]
fn test_iter() {
    let mut items = [1, 2, 3, 4, 5];
    let mut list: LinkedList<_> = items.into_iter().collect();

    assert!(list.iter().eq(&items));
    assert!(list.iter().rev().eq(items.iter().rev()));

    assert!(list.iter_mut().eq(&mut items));
    assert!(list.iter_mut().rev().eq(items.iter_mut().rev()));

    assert!(list.clone().into_iter().eq(items));
    assert!(list.clone().into_iter().rev().eq(items.into_iter().rev()));

    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.next_back(), Some(&5));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next_back(), Some(&4));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);

    let mut iter = list.iter_mut();
    assert_eq!(iter.next(), Some(&mut 1));
    assert_eq!(iter.next_back(), Some(&mut 5));
    assert_eq!(iter.next(), Some(&mut 2));
    assert_eq!(iter.next_back(), Some(&mut 4));
    assert_eq!(iter.next(), Some(&mut 3));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);

    let mut iter = list.into_iter();
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(5));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}
