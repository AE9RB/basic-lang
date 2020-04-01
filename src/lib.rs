//! # 64K BASIC
//!
//! The BASIC programming language as it was in the 8-bit era.
//!
//! Binaries for Windows and MacOS are available
//! [on GitHub.](https://github.com/AE9RB/basic-lang/releases)
//!
//! Linux requires [Rust](https://www.rust-lang.org/tools/install) then
//! the command `cargo install basic-lang`.
//!
//! Begin by opening a terminal and running the executable. Double clicking
//! the executable from a GUI desktop often works as well. If you get the
//! following, you have achieved success.
//! ```
//! 64K BASIC
//! READY.
//! █
//! ```
//!
//! A collection of BASIC games compatible with 64K BASIC is available
//! from [Vintage BASIC](http://vintage-basic.net/games.html).
//! These can be loaded with `LOAD "filename.bas"` then run with `RUN`.

#[path = "doc/introduction.rs"]
#[allow(non_snake_case)]
pub mod _Introduction;

#[path = "doc/chapter_1.rs"]
#[allow(non_snake_case)]
pub mod __Chapter_1;

#[path = "doc/chapter_2.rs"]
#[allow(non_snake_case)]
pub mod __Chapter_2;

#[path = "doc/chapter_3.rs"]
#[allow(non_snake_case)]
pub mod __Chapter_3;

#[path = "doc/appendix_a.rs"]
#[allow(non_snake_case)]
pub mod ___Appendix_A;

pub mod lang;
pub mod mach;
