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

pub mod lang;
pub mod mach;
use lang::Line;
use mach::Program;
use mach::Runtime;

fn main() {
    let t = Line::new(" 10letg=1+3*3:goto10:remarkABLE! \r\n");
    println!("{}", t);
    println!("{:?}", t.ast());

    let mut p = Program::new();
    p.compile(&Line::new("0 letx=y"));
    p.link();
    println!("{:?}", p);

    let mut r = Runtime::new();
    r.enter(Line::new("10?\"hello\",1,2,3"));
    r.enter(Line::new("?999,:run"));
}
