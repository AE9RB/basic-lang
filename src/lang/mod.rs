/*!
## Rust Language Module

This Rust module provides lexical analysis and parsing of the BASIC language.

64K BASIC aims to be compatible with the BASIC that was popular among
8-bit computers in the 1970s and 80s. These BASIC interpreters would typically
occupy 4 to 10 kilobytes of an available 64k memory space. Every byte of RAM was
precious. BASIC programs were kept in memory using a trivial compression method
where known words would only occupy a single byte. This infallible tokenization
was the only thing done before a program was run. The interpreter used the token
stream as a crude VM instruction set.

64K BASIC performs context-sensitive lexical analysis and parses to an
abstract syntax tree (AST). The lex() and parse() functions are exposed but
it is recommended to use tokens() and ast() from Line instead.

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
