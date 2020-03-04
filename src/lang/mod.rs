/*!
## Rust Language Module

This Rust module provides lexical analysis and parsing of the BASIC language.

*/

pub type Column = std::ops::Range<usize>;
pub type LineNumber = Option<u16>;
pub trait MaxValue {
    fn max_value() -> u16;
}
impl MaxValue for LineNumber {
    fn max_value() -> u16 {
        65529
    }
}

mod error;
mod ident;
mod lex;
mod line;
mod parse;
mod token;

pub use error::Error;
pub use error::ErrorCode;
pub use lex::lex;
pub use line::Line;
pub use parse::parse;

pub mod ast;
