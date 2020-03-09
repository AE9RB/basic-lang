/*!
## Rust Language Module

This Rust module provides lexical analysis and parsing of the BASIC language.

*/

pub type Column = std::ops::Range<usize>;
pub type LineNumber = Option<u16>;
pub trait MaxValue<T> {
    fn max_value() -> T;
}
impl MaxValue<u16> for LineNumber {
    fn max_value() -> u16 {
        65529
    }
}

mod error;
mod ident;
mod lex;
mod line;
mod parse;

pub use error::Error;
pub use error::ErrorCode;
pub use lex::lex;
pub use line::Line;
pub use parse::parse;

pub mod ast;
pub mod token;
