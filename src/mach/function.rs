use super::{Opcode, Val};
use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Function {}

impl Function {
    pub fn opcode_and_arity(func_name: &str) -> Option<(Opcode, std::ops::RangeInclusive<usize>)> {
        match func_name {
            "COS" => Some((Opcode::Cos, 1..=1)),
            "SIN" => Some((Opcode::Sin, 1..=1)),
            _ => None,
        }
    }

    pub fn cos(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).cos())),
            Single(n) => Ok(Single(n.cos())),
            Double(n) => Ok(Double(n.cos())),
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }
    pub fn sin(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).sin())),
            Single(n) => Ok(Single(n.sin())),
            Double(n) => Ok(Double(n.sin())),
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
