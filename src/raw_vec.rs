#![allow(unsafe_code)]

use alloc::{alloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};

pub struct RawVec<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
}

impl<T> RawVec<T> {
    pub const START_CAPACITY: usize = match mem::size_of::<T>() {
        1 => 8,
        ..=1024 => 4,
        _ => 1,
    };
    #[must_use]
    pub const fn new() -> Self {
        Self { ptr: NonNull::dangling(), cap: 0 }
    }
    pub fn grow(&mut self) {
        self.reserve(1);
    }
    pub fn reserve(&mut self, additional: usize) {
        let new_cap = if self.cap == 0 { Self::START_CAPACITY } else { 2 * self.cap };
        let new_cap = new_cap.max(self.cap + additional);
        self.resize(new_cap);
    }
    /// # Panics
    /// Panics if `new_cap * size_of::<T> > isize::MAX`
    pub fn resize(&mut self, new_cap: usize) {
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        assert!(isize::try_from(new_layout.size()).is_ok(), "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr().cast();
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = NonNull::new(new_ptr)
            .map_or_else(|| alloc::handle_alloc_error(new_layout), NonNull::cast);
        self.cap = new_cap;
    }
    /// # Safety
    /// Read `core::ptr::write`;
    pub unsafe fn write(&mut self, index: usize, val: T) {
        unsafe {
            ptr::write(self.ptr.as_ptr().add(index), val);
        }
    }
    /// # Safety
    /// Read `core::ptr::read`;
    pub unsafe fn read(&mut self, index: usize) -> T {
        unsafe { ptr::read(self.ptr.as_ptr().add(index)) }
    }
    /// # Safety
    /// Read `core::ptr::copy`;
    pub unsafe fn shift(&mut self, from: usize, to: usize, count: usize) {
        ptr::copy(self.ptr.as_ptr().add(from), self.ptr.as_ptr().add(to), count);
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if mem::size_of::<T>() == 0 || self.cap == 0 {
            return;
        }
        unsafe {
            alloc::dealloc(self.ptr.as_ptr().cast(), Layout::array::<T>(self.cap).unwrap());
        }
    }
}

impl<T> Default for RawVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Send for RawVec<T> where T: Send {}
unsafe impl<T> Sync for RawVec<T> where T: Sync {}
