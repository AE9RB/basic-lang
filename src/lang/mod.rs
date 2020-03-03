/*!
# Rust Language Module

This Rust module provides lexical analysis and parsing of the BASIC language.

*/

#[macro_use]
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
