use super::{Opcode, Val};
use crate::error;
use crate::lang::Error;
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

pub struct Function {}

impl Function {
    pub fn opcode_and_arity(func_name: &str) -> Option<(Opcode, std::ops::RangeInclusive<usize>)> {
        match func_name {
            "COS" => Some((Opcode::Cos, 1..=1)),
            "SIN" => Some((Opcode::Sin, 1..=1)),
            "TAB" => Some((Opcode::Tab, 1..=1)),
            _ => None,
        }
    }

    pub fn cos(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).cos())),
            Single(n) => Ok(Single(n.cos())),
            Double(n) => Ok(Double(n.cos())),
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }
    pub fn sin(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).sin())),
            Single(n) => Ok(Single(n.sin())),
            Double(n) => Ok(Double(n.sin())),
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn tab(val: Val, print_col: usize) -> Result<Val> {
        let mut s = String::new();
        let tab = i16::try_from(val)?;
        if tab < -255 || tab > 255 {
            return Err(error!(Overflow));
        }
        if tab < 0 {
            let tab = -tab as usize;
            let len = tab - (print_col % tab);
            s.push_str(&" ".repeat(len));
        } else if tab as usize > print_col {
            let len = tab as usize - print_col;
            s.push_str(&" ".repeat(len));
        }
        Ok(Val::String(s))
    }
}
