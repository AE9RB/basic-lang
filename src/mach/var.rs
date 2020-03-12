use super::Val;
use crate::error;
use crate::lang::Error;
use std::collections::HashMap;
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

/// ## Variable memory

#[derive(Debug)]
pub struct Var {
    vars: HashMap<String, Val>,
}

impl Var {
    pub fn new() -> Var {
        Var {
            vars: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vars.clear()
    }

    pub fn fetch(&self, var_name: &str) -> Val {
        match self.vars.get(var_name) {
            Some(val) => val.clone(),
            None => {
                if var_name.ends_with("$") {
                    Val::String("".to_string())
                } else if var_name.ends_with("!") {
                    Val::Single(0.0)
                } else if var_name.ends_with("#") {
                    Val::Double(0.0)
                } else if var_name.ends_with("%") {
                    Val::Integer(0)
                } else {
                    Val::Single(0.0)
                }
            }
        }
    }

    pub fn store(&mut self, var_name: &str, value: Val) -> Result<()> {
        if self.vars.len() > u16::max_value() as usize {
            return Err(error!(OutOfMemory));
        }
        if var_name.ends_with('$') {
            self.insert_string(var_name, value)
        } else if var_name.ends_with('!') {
            self.insert_single(var_name, value)
        } else if var_name.ends_with('#') {
            self.insert_double(var_name, value)
        } else if var_name.ends_with('%') {
            self.insert_integer(var_name, value)
        } else {
            self.insert_single(var_name, value)
        }
    }

    fn insert_val(&mut self, var_name: &str, value: Val) {
        match self.vars.get_mut(var_name) {
            Some(var) => *var = value,
            None => {
                self.vars.insert(var_name.to_string(), value);
            }
        };
    }

    fn insert_string(&mut self, var_name: &str, value: Val) -> Result<()> {
        match &value {
            Val::String(s) => {
                if s.chars().count() > 255 {
                    return Err(error!(OutOfStringSpace; "MAXIMUM STRING LENGTH IS 255"));
                }
                Ok(self.insert_val(var_name, value))
            }
            _ => Err(error!(TypeMismatch)),
        }
    }

    fn insert_integer(&mut self, var_name: &str, value: Val) -> Result<()> {
        match value {
            Val::Integer(_) => Ok(self.insert_val(var_name, value)),
            _ => Ok(self.insert_val(var_name, Val::Integer(i16::try_from(value)?))),
        }
    }

    fn insert_single(&mut self, var_name: &str, value: Val) -> Result<()> {
        match value {
            Val::Single(_) => Ok(self.insert_val(var_name, value)),
            _ => Ok(self.insert_val(var_name, Val::Single(f32::try_from(value)?))),
        }
    }

    fn insert_double(&mut self, var_name: &str, value: Val) -> Result<()> {
        match value {
            Val::Double(_) => Ok(self.insert_val(var_name, value)),
            _ => Ok(self.insert_val(var_name, Val::Double(f64::try_from(value)?))),
        }
    }
}
