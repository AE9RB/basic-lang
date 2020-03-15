use super::{Address, Op, Stack, Symbol};
use crate::error;
use crate::lang::{Column, Error, LineNumber, MaxValue};
use std::collections::{BTreeMap, HashMap};

type Result<T> = std::result::Result<T, Error>;

/// ## Link editor

#[derive(Debug, Default)]
pub struct Link {
    current_symbol: Symbol,
    symbols: BTreeMap<Symbol, Address>,
    unlinked: HashMap<Address, (Column, Symbol)>,
    loops: Vec<(Column, String, Symbol, Symbol)>,
}

impl Link {
    pub fn new() -> Link {
        Link::default()
    }
    pub fn clear(&mut self) {
        self.current_symbol = 0;
        self.symbols.clear();
        self.unlinked.clear();
    }

    pub fn next_symbol(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.current_symbol
    }

    pub fn insert(&mut self, sym: Symbol, addr: Address) {
        self.symbols.insert(sym, addr);
    }

    pub fn symbol_for_line_number(&mut self, line_number: LineNumber) -> Result<Symbol> {
        match line_number {
            Some(number) => Ok(number as Symbol),
            None => Err(error!(InternalError; "NO SYMBOL FOR LINE NUMBER")),
        }
    }

    pub fn link_addr_to_symbol(&mut self, addr: Address, col: Column, symbol: Symbol) {
        self.unlinked.insert(addr, (col, symbol));
    }

    pub fn begin_for_loop(&mut self, addr: Address, col: Column, var_name: String) -> Result<()> {
        let loop_start = self.next_symbol();
        let loop_end = self.next_symbol();
        self.loops
            .push((col.clone(), var_name, loop_start, loop_end));
        self.insert(loop_start, addr);
        self.link_addr_to_symbol(addr, col, loop_end);
        Ok(())
    }

    pub fn next_for_loop(&mut self, addr: Address, col: Column, var_name: String) -> Result<()> {
        if let Some((_col, for_name, loop_start, loop_end)) = self.loops.pop() {
            if var_name.is_empty() || var_name == for_name {
                self.link_addr_to_symbol(addr, col, loop_start);
                self.insert(loop_end, addr + 1);
                return Ok(());
            }
        }
        Err(error!(NextWithoutFor, ..&col))
    }

    pub fn set_start_of_direct(&mut self, op_addr: Address) {
        self.insert(LineNumber::max_value() as isize + 1 as Symbol, op_addr);
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        for (line_number, symbol_addr) in self.symbols.range(0..).rev() {
            if op_addr >= *symbol_addr && *line_number <= LineNumber::max_value() as isize {
                return Some(*line_number as u16);
            }
        }
        None
    }

    pub fn link(&mut self, ops: &mut Stack<Op>) -> Vec<Error> {
        let mut errors: Vec<Error> = vec![];
        for (col, _, loop_start, _) in std::mem::take(&mut self.loops) {
            let line_number = match self.symbols.get(&loop_start) {
                None => None,
                Some(addr) => {
                    self.unlinked.remove(addr);
                    self.line_number_for(*addr)
                }
            };
            errors.push(error!(ForWithoutNext, line_number, ..&col));
        }
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
                    if let Some(op) = ops.get_mut(op_addr) {
                        if let Some(new_op) = match op {
                            Op::For(_) => Some(Op::For(*dest)),
                            Op::If(_) => Some(Op::If(*dest)),
                            Op::Jump(_) => Some(Op::Jump(*dest)),
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
