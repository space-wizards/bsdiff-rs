//! Bsdiff is a method of diffing files.
//! This crate has been ported from C code.
//! The original code and more info can be found [here](https://github.com/mendsley/bsdiff).
//!
//! It is usually a good idea to use bsdiff alongside a compression algorithm like bzip2.
//!
//! # Example
//!
//! ```
//! let one = vec![1, 2, 3, 4, 5];
//! let two = vec![1, 2, 4, 6];
//! let mut patch = Vec::new();
//!
//! bsdiff::diff(&one, &two, &mut patch).unwrap();
//!
//! let mut patched = Vec::with_capacity(two.len());
//! bsdiff::patch(&one, &mut patch.as_slice(), &mut patched).unwrap();
//! assert_eq!(patched, two);
//! ```

mod diff;
mod patch;

pub use diff::diff;
pub use patch::patch;
