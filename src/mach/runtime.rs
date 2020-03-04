use super::{Address, Program, Val};

#[allow(dead_code)]
pub struct Runtime {
    vars: std::collections::HashMap<String, Val>,
    stack: Vec<Val>,
    program: Program,
    program_counter: Address,
}
