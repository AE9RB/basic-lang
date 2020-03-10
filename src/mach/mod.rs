/*!
## Rust Machine Module

This Rust module is a compiler and virtual machine for BASIC.

*/

pub type Address = usize;
pub type Symbol = isize;

mod compile;
mod op;
mod program;
mod runtime;
mod stack;
mod val;

pub use op::Op;
pub use program::Program;
pub use runtime::Event;
pub use runtime::Runtime;
pub use stack::Stack;
pub use val::Val;
