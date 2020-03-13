use super::Address;
use crate::error;
use crate::lang::{Error, LineNumber, MaxValue};
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

/// ## Runtime values for stack and variables

#[derive(Debug, Clone)]
pub enum Val {
    String(String),
    Single(f32),
    Double(f64),
    Integer(i16),
    Char(char),
    Return(Address),
}

impl Val {
    pub fn unimplemented(_lhs: Val, _rhs: Val) -> Result<Val> {
        Err(error!(InternalError; "OP NOT IMPLEMENTED; PANIC"))
    }

    pub fn neg(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(l) => Ok(Integer(-l)),
            Single(l) => Ok(Single(-l)),
            Double(l) => Ok(Double(-l)),
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn multiply(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        loop {
            return match lhs {
                Integer(l) => match rhs {
                    Integer(r) => Ok(Integer({
                        match l.checked_mul(r) {
                            Some(a) => a,
                            None => return Err(error!(Overflow)),
                        }
                    })),
                    Single(r) => Ok(Single(l as f32 * r)),
                    Double(r) => Ok(Double(l as f64 * r)),
                    _ => break,
                },
                Single(l) => match rhs {
                    Integer(r) => Ok(Single(l * r as f32)),
                    Single(r) => Ok(Single(l * r)),
                    Double(r) => Ok(Double(l as f64 * r)),
                    _ => break,
                },
                Double(l) => match rhs {
                    Integer(r) => Ok(Double(l * r as f64)),
                    Single(r) => Ok(Double(l * r as f64)),
                    Double(r) => Ok(Double(l * r)),
                    _ => break,
                },
                String(_) | Char(_) | Return(_) => break,
            };
        }
        Err(error!(TypeMismatch))
    }

    pub fn divide(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        loop {
            return match lhs {
                Integer(l) => match rhs {
                    Integer(r) => Ok(Integer({
                        match l.checked_div(r) {
                            Some(a) => a,
                            None => {
                                if r == 0 {
                                    return Err(error!(DivisionByZero));
                                };
                                return Err(error!(Overflow));
                            }
                        }
                    })),
                    Single(r) => Ok(Single(l as f32 / r)),
                    Double(r) => Ok(Double(l as f64 / r)),
                    _ => break,
                },
                Single(l) => match rhs {
                    Integer(r) => Ok(Single(l / r as f32)),
                    Single(r) => Ok(Single(l / r)),
                    Double(r) => Ok(Double(l as f64 / r)),
                    _ => break,
                },
                Double(l) => match rhs {
                    Integer(r) => Ok(Double(l / r as f64)),
                    Single(r) => Ok(Double(l / r as f64)),
                    Double(r) => Ok(Double(l / r)),
                    _ => break,
                },
                String(_) | Char(_) | Return(_) => break,
            };
        }
        Err(error!(TypeMismatch))
    }

    pub fn add(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        loop {
            return match lhs {
                String(mut l) => match rhs {
                    String(r) => Ok(String(l + &r)),
                    Char(r) => Ok(String({
                        l.push(r);
                        l
                    })),
                    _ => break,
                },
                Char(l) => match rhs {
                    String(r) => Ok(String(l.to_string() + &r)),
                    Char(r) => Ok(String({
                        let mut l = l.to_string();
                        l.push(r);
                        l
                    })),
                    _ => break,
                },
                Integer(l) => match rhs {
                    Integer(r) => Ok(Integer({
                        match l.checked_add(r) {
                            Some(a) => a,
                            None => return Err(error!(Overflow)),
                        }
                    })),
                    Single(r) => Ok(Single(l as f32 + r)),
                    Double(r) => Ok(Double(l as f64 + r)),
                    _ => break,
                },
                Single(l) => match rhs {
                    Integer(r) => Ok(Single(l + r as f32)),
                    Single(r) => Ok(Single(l + r)),
                    Double(r) => Ok(Double(l as f64 + r)),
                    _ => break,
                },
                Double(l) => match rhs {
                    Integer(r) => Ok(Double(l + r as f64)),
                    Single(r) => Ok(Double(l + r as f64)),
                    Double(r) => Ok(Double(l + r)),
                    _ => break,
                },
                Return(_) => break,
            };
        }
        Err(error!(TypeMismatch))
    }

    pub fn subtract(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        loop {
            return match lhs {
                Integer(l) => match rhs {
                    Integer(r) => Ok(Integer({
                        match l.checked_sub(r) {
                            Some(a) => a,
                            None => return Err(error!(Overflow)),
                        }
                    })),
                    Single(r) => Ok(Single(l as f32 - r)),
                    Double(r) => Ok(Double(l as f64 - r)),
                    _ => break,
                },
                Single(l) => match rhs {
                    Integer(r) => Ok(Single(l - r as f32)),
                    Single(r) => Ok(Single(l - r)),
                    Double(r) => Ok(Double(l as f64 - r)),
                    _ => break,
                },
                Double(l) => match rhs {
                    Integer(r) => Ok(Double(l - r as f64)),
                    Single(r) => Ok(Double(l - r as f64)),
                    Double(r) => Ok(Double(l - r)),
                    _ => break,
                },
                String(_) | Char(_) | Return(_) => break,
            };
        }
        Err(error!(TypeMismatch))
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Val::*;
        match self {
            String(s) => write!(f, "{}", s),
            Integer(n) => write!(f, "{}", n),
            Single(n) => write!(f, "{}", n),
            Double(n) => write!(f, "{}", n),
            Char(c) => write!(f, "{}", c),
            Return(..) => {
                debug_assert!(false);
                write!(f, "")
            }
        }
    }
}

impl TryFrom<LineNumber> for Val {
    type Error = Error;
    fn try_from(line_number: LineNumber) -> std::result::Result<Self, Self::Error> {
        match line_number {
            Some(number) => Ok(Val::Single(number as f32)),
            None => Err(error!(UndefinedLine)),
        }
    }
}

impl TryFrom<Val> for LineNumber {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        let num = u16::try_from(val)?;
        if num <= LineNumber::max_value() {
            Ok(Some(num))
        } else {
            Err(error!(UndefinedLine))
        }
    }
}

impl TryFrom<Val> for u16 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => {
                if i >= 0 {
                    Ok(i as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(f) => {
                if f >= 0.0 && f <= u16::max_value() as f32 {
                    Ok(f as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= 0.0 && d <= u16::max_value() as f64 {
                    Ok(d as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for i16 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i),
            Val::Single(f) => {
                if f >= i16::min_value() as f32 && f <= i16::max_value() as f32 {
                    Ok(f as i16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= i16::min_value() as f64 && d <= i16::max_value() as f64 {
                    Ok(d as i16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for f32 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i as f32),
            Val::Single(f) => Ok(f),
            Val::Double(d) => Ok(d as f32),
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for f64 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i as f64),
            Val::Single(f) => Ok(f as f64),
            Val::Double(d) => Ok(d),
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
