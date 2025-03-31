#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

mod diff;
mod patch;

pub use diff::diff;
pub use patch::patch;
