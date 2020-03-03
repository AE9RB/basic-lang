use super::op::Address;
use super::program::Program;
use super::val::Val;

struct Runtime {
    vars: std::collections::HashMap<String, Val>,
    stack: Vec<Val>,
    program: Program,
    program_counter: Address,
}
