/*!
## Rust Terminal Module

Because BASIC must be interactive.

*/

// /* linefeed = "0.6" */
mod linefeed;
pub use crate::term::linefeed::main;

// /* rustyline = "6" */
// mod rustyline;
// pub use crate::term::rustyline::main;
