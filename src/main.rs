//! # BASIC
//!
//! The BASIC programming language as it was in 1978.
//!

mod lang;
use lang::line::*;

fn main() {
    let mut t = Line::new(" 10?10:fori=j%to10:g=1+3+sin(3):remark \r\n");
    println!("{}", t);
    println!("{:?}", t.ast());
}
