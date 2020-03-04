/*!
## Rust Machine Module

This Rust module is a process virtual machine for BASIC.

*/

pub type Address = usize;
pub type Symbol = isize;

mod compile;
mod op;
mod program;
mod runtime;
mod val;

pub use compile::compile;
pub use op::Op;
pub use program::Program;
pub use runtime::Runtime;
pub use val::Val;
