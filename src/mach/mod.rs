/*!
## Rust Machine Module

This Rust module is a compiler and virtual machine for BASIC.

*/

const MAX_LINE_LEN: usize = 1024;

pub type Address = usize;
pub type Symbol = isize;

mod codegen;
mod function;
mod link;
mod listing;
mod opcode;
mod operation;
mod program;
mod runtime;
mod stack;
mod val;
mod var;

pub use function::Function;
pub use link::Link;
pub use listing::Listing;
pub use opcode::Opcode;
pub use operation::Operation;
pub use program::Program;
pub use runtime::Event;
pub use runtime::Runtime;
pub use stack::Stack;
pub use val::Val;
pub use var::Var;
