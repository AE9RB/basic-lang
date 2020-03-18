//! # BASIC
//!
//! See lib for documentation.
//!

#![warn(clippy::all)]

pub mod lang;
pub mod mach;
mod term;

fn main() {
    term::main();
}
