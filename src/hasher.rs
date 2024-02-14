#![forbid(unsafe_code)]
// FIXME: Replace SipHasher with my own hasher.
#![allow(deprecated)]

use core::hash::{BuildHasherDefault, SipHasher};

#[allow(clippy::module_name_repetitions)]
pub type PlainBuildHasher = BuildHasherDefault<SipHasher>;
