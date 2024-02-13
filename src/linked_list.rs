// FIXME: Replace with forbid when possible.
#![deny(unsafe_code)]

use core::fmt;

use crate::vec::Vec;

type Idx = usize;
const NIL: usize = Idx::MAX;

#[derive(Clone)]
pub struct LinkedList<T> {
    buf: Vec<Node<T>>,
    head: Idx,
    tail: Idx,
}

impl<T> LinkedList<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: Vec::new(), head: NIL, tail: NIL }
    }

    pub fn push_back(&mut self, val: T) {
        let ptr = self.push_buf(Node { val, prev: self.tail, next: NIL });
        if self.head == NIL {
            self.head = ptr;
        }
        if self.tail != NIL {
            self.buf[self.tail].next = ptr;
        }
        self.tail = ptr;
    }

    pub fn push_front(&mut self, val: T) {
        let ptr = self.push_buf(Node { val, next: self.head, prev: NIL });
        if self.tail == NIL {
            self.tail = ptr;
        }
        if self.head != NIL {
            self.buf[self.head].prev = ptr;
        }
        self.head = ptr;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let node = self.remove_node(self.tail);
        self.tail = node.prev;
        if self.tail != NIL {
            self.buf[self.tail].next = NIL;
        }
        Some(node.val)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let node = self.remove_node(self.head);
        self.head = node.next;
        if self.head != NIL {
            self.buf[self.head].prev = NIL;
        }
        Some(node.val)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    fn remove_node(&mut self, ptr: Idx) -> Node<T> {
        let end = &self.buf[self.len() - 1];
        let (end_prev, end_next) = (end.prev, end.next);

        if let Some(prev) = self.buf.get_mut(end_prev) {
            prev.next = ptr;
        }
        if let Some(next) = self.buf.get_mut(end_next) {
            next.prev = ptr;
        }
        let node = self.buf.swap_remove(ptr);

        if self.head == self.len() {
            self.head = ptr;
        }
        if self.tail == self.len() {
            self.tail = ptr;
        }
        node
    }

    fn push_buf(&mut self, node: Node<T>) -> Idx {
        let ptr = self.buf.len();
        self.buf.push(node);
        ptr
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// IntoIter
pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn count(self) -> usize {
        self.len()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}
// Iter
pub struct Iter<'a, T> {
    list: &'a LinkedList<T>,
    head: Idx,
    tail: Idx,
    len: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let val = &self.list.buf[self.head].val;
        self.head = self.list.buf[self.head].next;
        Some(val)
    }

    fn count(self) -> usize {
        self.len()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let val = &self.list.buf[self.tail].val;
        self.tail = self.list.buf[self.tail].prev;
        Some(val)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        Iter { list: self, head: self.head, tail: self.tail, len: self.len() }
    }
}

// IterMut
pub struct IterMut<'a, T> {
    list: &'a mut LinkedList<T>,
    head: Idx,
    tail: Idx,
    len: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let head = self.head;
        self.head = self.list.buf[self.head].next;
        let val = &mut self.list.buf[head].val;

        // FIXME: remove this shit.
        #[allow(unsafe_code)]
        Some(unsafe { std::mem::transmute(val) })
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.len = self.len.checked_sub(1)?;
        let tail = self.tail;
        self.tail = self.list.buf[self.tail].prev;
        let val = &mut self.list.buf[tail].val;

        // FIXME: remove this shit.
        #[allow(unsafe_code)]
        Some(unsafe { std::mem::transmute(val) })
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        IterMut { head: self.head, tail: self.tail, len: self.len(), list: self }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
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

impl<T> LinkedList<T> {
    #[must_use]
    pub fn into_iter_unordered(self) -> impl ExactSizeIterator + DoubleEndedIterator {
        self.buf.into_iter().map(|node| node.val)
    }
}

impl<T> LinkedList<T> {
    #[must_use]
    pub fn iter_unordered(&self) -> impl ExactSizeIterator + DoubleEndedIterator + '_ {
        self.buf.iter().map(|node| &node.val)
    }
}

impl<T> LinkedList<T> {
    pub fn iter_mut_unordered(&mut self) -> impl ExactSizeIterator + DoubleEndedIterator + '_ {
        self.buf.iter_mut().map(|node| &mut node.val)
    }
}

impl<T> fmt::Debug for LinkedList<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

#[derive(Clone)]
pub struct Node<T> {
    val: T,
    next: Idx,
    prev: Idx,
}

#[test]
fn test_basics() {
    let mut list = LinkedList::new();
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
