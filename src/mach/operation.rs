use super::Val;
use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Operation {}

impl Operation {
    pub fn unimplemented(_lhs: Val, _rhs: Val) -> Result<Val> {
        Err(error!(InternalError; "OP NOT IMPLEMENTED; PANIC"))
    }

    pub fn negate(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(-n)),
            Single(n) => Ok(Single(-n)),
            Double(n) => Ok(Double(-n)),
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn multiply(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_mul(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 * r)),
                Double(r) => Ok(Double(l as f64 * r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l * r as f32)),
                Single(r) => Ok(Single(l * r)),
                Double(r) => Ok(Double(l as f64 * r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l * r as f64)),
                Single(r) => Ok(Double(l * r as f64)),
                Double(r) => Ok(Double(l * r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn divide(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_div(r) {
                    Some(i) => Ok(Integer(i)),
                    None => {
                        if r == 0 {
                            Err(error!(DivisionByZero))
                        } else {
                            Err(error!(Overflow))
                        }
                    }
                },
                Single(r) => Ok(Single(l as f32 / r)),
                Double(r) => Ok(Double(l as f64 / r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l / r as f32)),
                Single(r) => Ok(Single(l / r)),
                Double(r) => Ok(Double(l as f64 / r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l / r as f64)),
                Single(r) => Ok(Double(l / r as f64)),
                Double(r) => Ok(Double(l / r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn sum(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            String(mut l) => match rhs {
                String(r) => Ok(String(l + &r)),
                Char(r) => Ok(String({
                    l.push(r);
                    l
                })),
                _ => Err(error!(TypeMismatch)),
            },
            Char(l) => match rhs {
                String(r) => Ok(String(l.to_string() + &r)),
                Char(r) => Ok(String({
                    let mut l = l.to_string();
                    l.push(r);
                    l
                })),
                _ => Err(error!(TypeMismatch)),
            },
            Integer(l) => match rhs {
                Integer(r) => match l.checked_add(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 + r)),
                Double(r) => Ok(Double(l as f64 + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l + r as f32)),
                Single(r) => Ok(Single(l + r)),
                Double(r) => Ok(Double(l as f64 + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l + r as f64)),
                Single(r) => Ok(Double(l + r as f64)),
                Double(r) => Ok(Double(l + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn subtract(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_sub(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 - r)),
                Double(r) => Ok(Double(l as f64 - r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l - r as f32)),
                Single(r) => Ok(Single(l - r)),
                Double(r) => Ok(Double(l as f64 - r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l - r as f64)),
                Single(r) => Ok(Double(l - r as f64)),
                Double(r) => Ok(Double(l - r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn greater(lhs: Val, rhs: Val) -> Result<Val> {
        Operation::less(rhs, lhs)
    }

    pub fn less(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => {
                    if l < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Single(r) => {
                    if (l as f32) < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Double(r) => {
                    if (l as f64) < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => {
                    if l < r as f32 {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Single(r) => {
                    if l < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Double(r) => {
                    if (l as f64) < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => {
                    if l < r as f64 {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Single(r) => {
                    if l < r as f64 {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                Double(r) => {
                    if l < r {
                        Ok(Integer(-1))
                    } else {
                        Ok(Integer(0))
                    }
                }
                _ => Err(error!(TypeMismatch)),
            },
            Char(_) | String(_) => Err(error!(InternalError; "string compare not imp")),
            Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
