use super::op::Op;
use super::val::Val;

pub type Address = usize;

struct Runtime {
    vars: std::collections::HashMap<String, Val>,
    stack: Vec<Val>,
    program: Vec<Op>,
    program_counter: Address,
}
