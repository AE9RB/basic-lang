//! # 64K BASIC
//! ```text
//! READY.
//! █
//! ```
//! This is the manual. For binaries and games, start here: <http://basic-lang.org>
//!
//! 64K BASIC is compatible with programs from the beginning of personal computing.
//! It is designed to capture and preserve the best parts of the BASIC experience.
//! Getting a programming manual with your new computer hardware is best.
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
