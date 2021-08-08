# bsdiff-rs

[![Build status](https://github.com/space-wizards/bsdiff-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/space-wizards/bsdiff-rs/actions/workflows/rust.yml)
[![Cargo Link](https://img.shields.io/crates/v/bsdiff.svg)](https://crates.rs/crates/bsdiff)

Rust port of a [bsdiff library](https://github.com/mendsley/bsdiff). High performance patching. All written in safe Rust.

## Diffing

```rust
let old = std::fs::read("old")?;
let new = std::fs::read("new")?;
let mut patch = Vec::new();

bsdiff::diff(&old, &new, &mut patch)?;
// TODO: compress `patch` here
std::fs::write("patch", &patch)?;
```

## Patching

```rust
let old = std::fs::read("old")?;
let patch = std::fs::read("patch")?;
// TODO: decompress `patch` here
let mut new = Vec::new();

bsdiff::patch(&old, &mut patch.as_slice(), &mut new)?;
std::fs::write("new", &new)?;
```
