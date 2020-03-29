use super::{Address, Opcode, Stack, Symbol, Val};
use crate::error;
use crate::lang::{Column, Error, LineNumber, MaxValue};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

const OVERFLOW_MESSAGE: &str = "PROGRAM TOO LARGE";

/// ## Linkable object

#[derive(Debug)]
pub struct Link {
    current_symbol: Symbol,
    ops: Stack<Opcode>,
    symbols: BTreeMap<Symbol, Address>,
    unlinked: HashMap<Address, (Column, Symbol)>,
}

impl Default for Link {
    fn default() -> Self {
        Link {
            current_symbol: 0,
            ops: Stack::new(OVERFLOW_MESSAGE),
            symbols: BTreeMap::default(),
            unlinked: HashMap::default(),
        }
    }
}

impl Link {
    pub fn append(&mut self, mut link: Link) -> Result<()> {
        let addr_offset = self.ops.len();
        let sym_offset = self.current_symbol;
        for (symbol, address) in link.symbols.iter() {
            let mut symbol = *symbol;
            if symbol < 0 {
                symbol += sym_offset
            }
            self.symbols.insert(symbol, *address + addr_offset);
        }
        for (address, (col, symbol)) in link.unlinked.iter() {
            let mut symbol = *symbol;
            if symbol < 0 {
                symbol += sym_offset
            }
            self.unlinked
                .insert(*address + addr_offset, (col.clone(), symbol));
        }
        self.current_symbol += link.current_symbol;
        self.ops.append(&mut link.ops)
    }

    pub fn push(&mut self, op: Opcode) -> Result<()> {
        self.ops.push(op)
    }

    pub fn get(&self, addr: Address) -> Option<&Opcode> {
        self.ops.get(addr)
    }

    pub fn last(&self) -> Option<&Opcode> {
        self.ops.last()
    }

    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, Opcode>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.ops.drain(range)
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn clear(&mut self) {
        self.current_symbol = 0;
        self.ops.clear();
        self.symbols.clear();
        self.unlinked.clear();
    }

    pub fn next_symbol(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.current_symbol
    }

    fn symbol_for_line_number(&mut self, line_number: LineNumber) -> Result<Symbol> {
        match line_number {
            Some(number) => Ok(number as Symbol),
            None => Err(error!(InternalError; "NO SYMBOL FOR LINE NUMBER")),
        }
    }

    pub fn push_def_fn(
        &mut self,
        col: Column,
        ident: Rc<str>,
        vars: Vec<Rc<str>>,
        expr_ops: Link,
    ) -> Result<()> {
        let len = Val::try_from(vars.len())?;
        self.push(Opcode::Literal(len))?;
        self.push(Opcode::Def(ident))?;
        let skip = self.next_symbol();
        self.push_jump(col, skip)?;
        for var in vars {
            self.push(Opcode::Pop(var))?;
        }
        self.append(expr_ops)?;
        self.push(Opcode::Return)?;
        self.push_symbol(skip);
        Ok(())
    }

    pub fn push_for(&mut self, col: Column) -> Result<()> {
        let next = self.next_symbol();
        self.unlinked.insert(self.ops.len(), (col, next));
        self.ops.push(Opcode::Literal(Val::Next(0)))?;
        self.push_symbol(next);
        Ok(())
    }

    pub fn push_gosub(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        let ret_sym = self.next_symbol();
        self.push_return_val(col.clone(), ret_sym)?;
        let line_number_sym = self.symbol_for_line_number(line_number)?;
        self.unlinked.insert(self.ops.len(), (col, line_number_sym));
        self.ops.push(Opcode::Jump(0))?;
        self.push_symbol(ret_sym);
        Ok(())
    }

    pub fn push_return_val(&mut self, col: Column, symbol: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, symbol));
        self.ops.push(Opcode::Literal(Val::Return(0)))
    }

    pub fn push_goto(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        let sym = self.symbol_for_line_number(line_number)?;
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_ifnot(&mut self, col: Column, sym: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.push(Opcode::IfNot(0))
    }

    pub fn push_jump(&mut self, col: Column, sym: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.push(Opcode::Jump(0))
    }

    pub fn push_run(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        self.ops.push(Opcode::Clear)?;
        if line_number.is_some() {
            let sym = self.symbol_for_line_number(line_number)?;
            self.unlinked.insert(self.ops.len(), (col, sym));
        }
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_symbol(&mut self, sym: Symbol) {
        if self.symbols.insert(sym, self.ops.len()).is_some() {
            debug_assert!(false, "Symbol already exists.");
        }
    }

    pub fn set_start_of_direct(&mut self, op_addr: Address) {
        self.symbols
            .insert(LineNumber::max_value() as isize + 1 as Symbol, op_addr);
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        for (line_number, symbol_addr) in self.symbols.range(0..).rev() {
            if op_addr >= *symbol_addr {
                if *line_number <= LineNumber::max_value() as isize {
                    return Some(*line_number as u16);
                } else {
                    return None;
                }
            }
        }
        None
    }

    pub fn link(&mut self) -> Vec<Error> {
        let mut errors: Vec<Error> = vec![];
        for (op_addr, (col, symbol)) in std::mem::take(&mut self.unlinked) {
            match self.symbols.get(&symbol) {
                None => {
                    if symbol >= 0 {
                        let error = error!(UndefinedLine, self.line_number_for(op_addr), ..&col);
                        errors.push(error);
                        continue;
                    }
                }
                Some(dest) => {
                    if let Some(op) = self.ops.get_mut(op_addr) {
                        if let Some(new_op) = match op {
                            Opcode::IfNot(_) => Some(Opcode::IfNot(*dest)),
                            Opcode::Jump(_) => Some(Opcode::Jump(*dest)),
                            Opcode::Literal(Val::Return(_)) => {
                                Some(Opcode::Literal(Val::Return(*dest)))
                            }
                            Opcode::Literal(Val::Next(_)) => {
                                Some(Opcode::Literal(Val::Next(*dest)))
                            }
                            _ => None,
                        } {
                            *op = new_op;
                            continue;
                        }
                    }
                }
            }
            let line_number = self.line_number_for(op_addr);
            errors.push(error!(InternalError, line_number, ..&col; "LINK FAILURE"));
        }
        self.symbols = self.symbols.split_off(&0);
        self.current_symbol = 0;
        errors
    }
}

impl TryFrom<Link> for LineNumber {
    type Error = Error;

    fn try_from(mut prog: Link) -> std::result::Result<Self, Self::Error> {
        if prog.ops.len() == 1 {
            match prog.ops.pop() {
                Ok(Opcode::Literal(val)) => return Ok(LineNumber::try_from(val)?),
                Err(e) => return Err(e),
                _ => {}
            }
        }
        Err(error!(UndefinedLine; "INVALID LINE NUMBER"))
    }
}
