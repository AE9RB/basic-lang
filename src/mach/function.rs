use super::{Opcode, Val};
use crate::error;
use crate::lang::Error;
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

pub struct Function {}

impl Function {
    pub fn opcode_and_arity(func_name: &str) -> Option<(Opcode, std::ops::RangeInclusive<usize>)> {
        match func_name {
            "ABS" => Some((Opcode::Abs, 1..=1)),
            "CHR$" => Some((Opcode::Chr, 1..=1)),
            "COS" => Some((Opcode::Cos, 1..=1)),
            "INT" => Some((Opcode::Int, 1..=1)),
            "RND" => Some((Opcode::Rnd, 0..=1)),
            "SIN" => Some((Opcode::Sin, 1..=1)),
            "TAB" => Some((Opcode::Tab, 1..=1)),
            _ => None,
        }
    }

    pub fn abs(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(n.abs())),
            Single(n) => Ok(Single(n.abs())),
            Double(n) => Ok(Double(n.abs())),
            String(_) | Return(_) | Val::Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn chr(val: Val) -> Result<Val> {
        match char::try_from(u32::try_from(val)?) {
            Ok(n) => Ok(Val::String(n.to_string().into())),
            Err(_) => Err(error!(Overflow)),
        }
    }

    pub fn cos(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).cos())),
            Single(n) => Ok(Single(n.cos())),
            Double(n) => Ok(Double(n.cos())),
            String(_) | Return(_) | Val::Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn int(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(n)),
            Single(n) => Ok(Single(n.floor())),
            Double(n) => Ok(Double(n.floor())),
            String(_) | Return(_) | Val::Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn rnd(st: &mut (u32, u32, u32), mut vec_val: Vec<Val>) -> Result<Val> {
        let val = match vec_val.pop() {
            Some(s) => f32::try_from(s)?,
            None => 1.0,
        };
        if val < 0.0 {
            let seed = u32::from_le_bytes(val.to_be_bytes()) & 0x_00FF_FFFF;
            st.0 = seed;
            st.1 = seed;
            st.2 = seed;
        }
        if val != 0.0 {
            st.0 = (171 * st.0) % 30269;
            st.1 = (172 * st.1) % 30307;
            st.2 = (170 * st.2) % 30323;
        }
        Ok(Val::Single(
            (st.0 as f32 / 30269.0 + st.1 as f32 / 30307.0 + st.2 as f32 / 30323.0) % 1.0,
        ))
    }

    pub fn sin(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).sin())),
            Single(n) => Ok(Single(n.sin())),
            Double(n) => Ok(Double(n.sin())),
            String(_) | Return(_) | Val::Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn tab(print_col: usize, val: Val) -> Result<Val> {
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
        Ok(Val::String(s.into()))
    }
}
