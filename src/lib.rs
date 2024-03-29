#![deny(unsafe_code)]
#![feature(impl_trait_in_assoc_type)]
#![no_std]

extern crate alloc;

pub mod hasher;
pub mod hashmap;
pub mod hashset;
pub mod linked_list;
pub(crate) mod raw_vec;
pub mod vec;

pub use hashmap::HashMap;
pub use hashset::HashSet;
pub use vec::Vec;
