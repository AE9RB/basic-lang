//! # BASIC
//!
//! The BASIC programming language as it was in 1978.
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

#[path = "doc/appendix_a.rs"]
#[allow(non_snake_case)]
pub mod ___Appendix_A;

mod lang;
mod mach;
use lang::line::*;
use mach::compile::*;

fn main() {
    let t = Line::new(" 10letg=1+3*3:remarkABLE! \r\n");
    println!("{}", t);
    println!("{:?}", t.ast());
    let c = compile(&t);
    println!("{:?}", c);
}
