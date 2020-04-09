//! # 64K BASIC
//!
//! The BASIC programming language as it was in the 8-bit era.
//! ```text
//! 64K BASIC
//! READY.
//! █
//! ```
//!
//! ## Installation
//!
//! Binaries for Windows and MacOS are available
//! [on GitHub.](https://github.com/AE9RB/basic-lang/releases)
//!
//! Linux requires [Rust](https://www.rust-lang.org/tools/install) then
//! the command `cargo install basic-lang`.
//!
//! ## Getting Started
//!
//! [The patch repository](https://github.com/AE9RB/basic-lang/tree/master/patch)
//! contains many programs that can be automatically downloaded and patched.
//!
//! This is the manual. Every type, statement, operation, and function is documented.
//! 64K BASIC is designed to capture and preserve the best parts of the BASIC experience.
//! Getting a programming manual with your computer was definitely best.
//!

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

#[path = "doc/appendix_b.rs"]
#[allow(non_snake_case)]
pub mod ___Appendix_B;

pub mod lang;
pub mod mach;
