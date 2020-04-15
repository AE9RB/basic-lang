extern crate chrono;
use super::{Opcode, Stack, Val};
use crate::error;
use crate::lang::Error;
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

pub struct Function {}

impl Function {
    pub fn opcode_and_arity(func_name: &str) -> Option<(Opcode, std::ops::RangeInclusive<usize>)> {
        match func_name {
            "ABS" => Some((Opcode::Abs, 1..=1)),
            "ASC" => Some((Opcode::Asc, 1..=1)),
            "ATN" => Some((Opcode::Atn, 1..=1)),
            "CDBL" => Some((Opcode::Cdbl, 1..=1)),
            "CHR$" => Some((Opcode::Chr, 1..=1)),
            "CINT" => Some((Opcode::Cint, 1..=1)),
            "COS" => Some((Opcode::Cos, 1..=1)),
            "CSNG" => Some((Opcode::Csng, 1..=1)),
            "DATE$" => Some((Opcode::Date, 0..=0)),
            "EXP" => Some((Opcode::Exp, 1..=1)),
            "FIX" => Some((Opcode::Fix, 1..=1)),
            "HEX$" => Some((Opcode::Hex, 1..=1)),
            "INKEY$" => Some((Opcode::Inkey, 0..=0)),
            "INSTR" => Some((Opcode::Instr, 2..=3)),
            "INT" => Some((Opcode::Int, 1..=1)),
            "LEFT$" => Some((Opcode::Left, 2..=2)),
            "LEN" => Some((Opcode::Len, 1..=1)),
            "LOG" => Some((Opcode::Log, 1..=1)),
            "MID$" => Some((Opcode::Mid, 2..=3)),
            "OCT$" => Some((Opcode::Oct, 1..=1)),
            "POS" => Some((Opcode::Pos, 0..=1)),
            "RIGHT$" => Some((Opcode::Right, 2..=2)),
            "RND" => Some((Opcode::Rnd, 0..=1)),
            "SGN" => Some((Opcode::Sgn, 1..=1)),
            "SIN" => Some((Opcode::Sin, 1..=1)),
            "SPC" => Some((Opcode::Spc, 1..=1)),
            "SQR" => Some((Opcode::Sqr, 1..=1)),
            "STR$" => Some((Opcode::Str, 1..=1)),
            "STRING$" => Some((Opcode::String, 2..=2)),
            "TAB" => Some((Opcode::Tab, 1..=1)),
            "TAN" => Some((Opcode::Tan, 1..=1)),
            "TIME$" => Some((Opcode::Time, 0..=0)),
            "VAL" => Some((Opcode::Val, 1..=1)),
            _ => None,
        }
    }

    pub fn abs(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(n.abs())),
            Single(n) => Ok(Single(n.abs())),
            Double(n) => Ok(Double(n.abs())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn asc(string: Val) -> Result<Val> {
        let string = Rc::<str>::try_from(string)?;
        match string.chars().next() {
            Some(ch) => {
                let num = u32::from(ch);
                if num <= i16::max_value() as u32 {
                    Ok(Val::Integer(num as i16))
                } else if num <= 16_777_216 {
                    Ok(Val::Single(num as f32))
                } else {
                    Ok(Val::Double(num as f64))
                }
            }
            None => Err(error!(IllegalFunctionCall)),
        }
    }

    pub fn atn(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).atan())),
            Single(n) => Ok(Single(n.atan())),
            Double(n) => Ok(Double(n.atan())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn cdbl(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Double(n as f64)),
            Single(n) => Ok(Double(n as f64)),
            Double(n) => Ok(Double(n)),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn chr(val: Val) -> Result<Val> {
        match char::try_from(u32::try_from(val)?) {
            Ok(ch) => Ok(Val::String(ch.to_string().into())),
            Err(_) => Err(error!(Overflow)),
        }
    }

    pub fn cint(val: Val) -> Result<Val> {
        Ok(Val::Integer(i16::try_from(val)?))
    }

    pub fn cos(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).cos())),
            Single(n) => Ok(Single(n.cos())),
            Double(n) => Ok(Double(n.cos())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn csng(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single(n as f32)),
            Single(n) => Ok(Single(n)),
            Double(n) => Ok(Single(n as f32)),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn date() -> Result<Val> {
        Ok(Val::String(
            chrono::Local::now().format("%m-%d-%Y").to_string().into(),
        ))
    }

    pub fn exp(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).exp())),
            Single(n) => Ok(Single(n.exp())),
            Double(n) => Ok(Double(n.exp())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn fix(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(n)),
            Single(n) => Ok(Single(n.trunc())),
            Double(n) => Ok(Double(n.trunc())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn hex(val: Val) -> Result<Val> {
        let num = i16::try_from(val)?;
        Ok(Val::String(format!("{:X}", num).into()))
    }

    pub fn instr(mut vec_val: Stack<Val>) -> Result<Val> {
        let pattern = Rc::<str>::try_from(vec_val.pop()?)?;
        let string = Rc::<str>::try_from(vec_val.pop()?)?;
        let start = match vec_val.pop() {
            Ok(n) => i16::try_from(n)?,
            Err(_) => 1,
        } as usize;
        if start == 0 {
            return Err(error!(IllegalFunctionCall; "START IS 0"));
        }
        let ch_idx = match string.char_indices().nth(start - 1) {
            Some((pos, _ch)) => pos,
            None => return Ok(Val::Integer(0)),
        };
        let string: Rc<str> = string[ch_idx..].into(); //??
        let index = match string.find(pattern.as_ref()) {
            Some(n) => n,
            None => 0,
        };
        let str_index = string
            .char_indices()
            .enumerate()
            .find_map(|(str_idx, (ch_idx, _ch))| {
                if ch_idx == index {
                    Some(str_idx + start)
                } else {
                    None
                }
            });
        match str_index {
            Some(n) => Ok(Val::try_from(n)?),
            None => Ok(Val::Integer(0)),
        }
    }

    pub fn int(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(n)),
            Single(n) => Ok(Single(n.floor())),
            Double(n) => Ok(Double(n.floor())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn left(string: Val, len: Val) -> Result<Val> {
        let len = usize::try_from(len)?;
        let string = Rc::<str>::try_from(string)?;
        match string.char_indices().nth(len) {
            Some((pos, _ch)) => Ok(Val::String(string[..pos].into())),
            None => Ok(Val::String(string)),
        }
    }

    pub fn len(string: Val) -> Result<Val> {
        let string = Rc::<str>::try_from(string)?;
        Ok(Val::try_from(string.chars().count())?)
    }

    pub fn log(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).ln())),
            Single(n) => Ok(Single(n.ln())),
            Double(n) => Ok(Double(n.ln())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn mid(mut args: Stack<Val>) -> Result<Val> {
        let len = match args.len() {
            3 => Some(u16::try_from(args.pop()?)?),
            _ => None,
        };
        let pos = usize::try_from(args.pop()?)?;
        if pos == 0 {
            return Err(error!(Overflow));
        }
        let string = Rc::<str>::try_from(args.pop()?)?;
        match string.char_indices().nth(pos - 1) {
            Some((pos, _ch)) => match len {
                None => Ok(Val::String(string[pos..].into())),
                Some(len) => {
                    let string: Rc<str> = string[pos..].into();
                    match string.char_indices().nth(len as usize) {
                        Some((pos, _ch)) => Ok(Val::String(string[..pos].into())),
                        None => Ok(Val::String(string)),
                    }
                }
            },
            None => Ok(Val::String(string)),
        }
    }

    pub fn oct(val: Val) -> Result<Val> {
        let num = i16::try_from(val)?;
        Ok(Val::String(format!("{:o}", num).into()))
    }

    pub fn pos(print_col: usize) -> Result<Val> {
        match i16::try_from(print_col) {
            Ok(pos) => Ok(Val::Integer(pos)),
            Err(_) => Err(error!(Overflow)),
        }
    }

    pub fn right(string: Val, len: Val) -> Result<Val> {
        let len = usize::try_from(len)?;
        if len == 0 {
            return Ok(Val::String("".into()));
        }
        let string = Rc::<str>::try_from(string)?;
        match string.char_indices().rev().nth(len - 1) {
            Some((pos, _ch)) => Ok(Val::String(string[pos..].into())),
            None => Ok(Val::String(string)),
        }
    }

    pub fn rnd(st: &mut (u32, u32, u32), mut vec_val: Stack<Val>) -> Result<Val> {
        let val = match vec_val.pop() {
            Ok(s) => f32::try_from(s)?,
            Err(_) => 1.0,
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

    pub fn sgn(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(if n == 0 {
                0
            } else if n.is_negative() {
                -1
            } else {
                1
            })),
            Single(n) => Ok(Integer(if n == 0.0 {
                0
            } else if n.is_sign_negative() {
                -1
            } else {
                1
            })),
            Double(n) => Ok(Integer(if n == 0.0 {
                0
            } else if n.is_sign_negative() {
                -1
            } else {
                1
            })),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn sin(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).sin())),
            Single(n) => Ok(Single(n.sin())),
            Double(n) => Ok(Double(n.sin())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn spc(val: Val) -> Result<Val> {
        let len = usize::try_from(val)?;
        if len > 255 {
            return Err(error!(Overflow));
        }
        Ok(Val::String(" ".repeat(len).into()))
    }

    pub fn sqr(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).sqrt())),
            Single(n) => Ok(Single(n.sqrt())),
            Double(n) => Ok(Double(n.sqrt())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn str(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(_) | Single(_) | Double(_) => Ok(String(format!("{}", val).into())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn string(num: Val, ch: Val) -> Result<Val> {
        let num = usize::try_from(num)?;
        if num > 255 {
            return Err(error!(Overflow));
        }
        let ch = match ch {
            Val::String(s) => match s.chars().next() {
                Some(ch) => ch,
                None => return Err(error!(IllegalFunctionCall)),
            },
            _ => {
                let num = u32::try_from(ch)?;
                match char::try_from(num) {
                    Ok(ch) => ch,
                    _ => return Err(error!(Overflow)),
                }
            }
        };
        Ok(Val::String(ch.to_string().repeat(num).into()))
    }

    pub fn tab(print_col: usize, val: Val) -> Result<Val> {
        let tab = i16::try_from(val)?;
        if tab < -255 || tab > 255 {
            return Err(error!(Overflow));
        }
        let len = if tab < 0 {
            let tab = -tab as usize;
            tab - (print_col % tab)
        } else if tab as usize > print_col {
            tab as usize - print_col
        } else {
            0
        };
        Ok(Val::String(" ".repeat(len).into()))
    }

    pub fn tan(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Single((n as f32).tan())),
            Single(n) => Ok(Single(n.tan())),
            Double(n) => Ok(Double(n.tan())),
            String(_) | Return(_) | Next(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn time() -> Result<Val> {
        Ok(Val::String(
            chrono::Local::now().format("%H:%M:%S").to_string().into(),
        ))
    }

    pub fn val(val: Val) -> Result<Val> {
        if let Val::String(s) = val {
            let mut s = s.trim();
            while let Some((idx, _)) = s.char_indices().last() {
                let v = Val::from(s);
                if !matches!(v, Val::String(_)) {
                    return Ok(v);
                }
                s = &s[0..idx];
            }
            Ok(Val::Integer(0))
        } else {
            Err(error!(TypeMismatch))
        }
    }
}
