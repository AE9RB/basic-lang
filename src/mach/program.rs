use super::{compile::compile, Address, Op, Stack, Symbol};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// ## Program memory

#[derive(Debug)]
pub struct Program {
    ops: Stack<Op>,
    errors: Arc<Vec<Error>>,
    indirect_errors: Arc<Vec<Error>>,
    direct_address: Address,
    current_symbol: Symbol,
    symbols: BTreeMap<Symbol, Address>,
    unlinked: HashMap<Address, (Column, Symbol)>,
    line_number: LineNumber,
}

impl Program {
    pub fn new() -> Program {
        Program {
            ops: Stack::new("PROGRAM TOO LARGE"),
            errors: Arc::new(vec![]),
            indirect_errors: Arc::new(vec![]),
            direct_address: 0,
            current_symbol: 0,
            symbols: BTreeMap::new(),
            unlinked: HashMap::new(),
            line_number: None,
        }
    }
    pub fn error(&mut self, error: Error) {
        Arc::make_mut(&mut self.errors).push(error.in_line_number(self.line_number));
    }
    pub fn append(&mut self, ops: &mut Stack<Op>) -> Result<(), Error> {
        self.ops.append(ops)
    }
    pub fn push(&mut self, op: Op) -> Result<(), Error> {
        self.ops.push(op)
    }
    pub fn ops(&self) -> &Vec<Op> {
        &self.ops.vec()
    }
    pub fn symbol_for_line_number(&mut self, line_number: LineNumber) -> Result<Symbol, Error> {
        match line_number {
            Some(number) => Ok(number as Symbol),
            None => Err(error!(InternalError; "NO SYMBOL FOR LINE NUMBER")),
        }
    }
    pub fn symbol_for_here(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.symbols.insert(self.current_symbol, self.ops.len());
        self.current_symbol
    }
    pub fn link_next_op_to(&mut self, col: &Column, symbol: Symbol) {
        self.unlinked.insert(self.ops.len(), (col.clone(), symbol));
    }
    pub fn clear(&mut self) {
        self.ops.clear();
        self.errors = Arc::new(vec![]);
        self.indirect_errors = Arc::new(vec![]);
        self.direct_address = 0;
        self.current_symbol = 0;
        self.symbols.clear();
        self.unlinked.clear();
        self.line_number = None;
    }
    pub fn compile<'b, T: IntoIterator<Item = &'b Line>>(&mut self, lines: T) {
        let is_out_of_mem = |this: &Self| this.ops.len() > Address::max_value() as usize;
        if is_out_of_mem(self) {
            return;
        }
        let mut direct_seen = false;
        for line in lines {
            if let Some(line_number) = line.number() {
                debug_assert!(
                    self.direct_address == 0,
                    "Can't go back to indirect mode without clear()."
                );
                if let Some(self_line_number) = self.line_number {
                    debug_assert!(line_number > self_line_number, "Lines out of order.");
                }
            } else {
                self.link();
            }
            self.line_number = line.number();
            if let Some(line_number) = self.line_number {
                self.symbols.insert(line_number as Symbol, self.ops.len());
            } else {
                debug_assert!(!direct_seen, "Can't handle multiple direct lines.");
                direct_seen = true;
                self.ops.drain(self.direct_address..);
                Arc::make_mut(&mut self.errors).clear();
            }
            let ast = match line.ast() {
                Ok(ast) => ast,
                Err(error) => {
                    Arc::make_mut(&mut self.errors).push(error);
                    continue;
                }
            };
            compile(self, &ast);
            if self.line_number.is_none() {
                if let Err(e) = self.ops.push(Op::End) {
                    Arc::make_mut(&mut self.errors).push(e);
                }
            }
            if is_out_of_mem(self) {
                Arc::make_mut(&mut self.errors).push(error!(OutOfMemory));
                return;
            }
        }
    }
    pub fn link(&mut self) -> (Address, Arc<Vec<Error>>, Arc<Vec<Error>>) {
        match self.ops.vec().last() {
            Some(Op::End) => {}
            _ => {
                if let Err(error) = self.ops.push(Op::End) {
                    Arc::make_mut(&mut self.errors).push(error);
                }
            }
        };
        for (op_addr, (col, symbol)) in std::mem::take(&mut self.unlinked) {
            match self.symbols.get(&symbol) {
                None => {
                    if symbol >= 0 {
                        let error = error!(UndefinedLine, self.line_number_for(op_addr), ..&col);
                        Arc::make_mut(&mut self.errors).push(error);
                        continue;
                    }
                }
                Some(dest) => {
                    if let Some(op) = self.ops.get_mut(op_addr) {
                        if let Some(new_op) = match op {
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
            Arc::make_mut(&mut self.errors)
                .push(error!(InternalError, line_number, ..&col; "LINK FAILURE"));
        }
        self.symbols = self.symbols.split_off(&0);
        self.current_symbol = 0;
        if self.direct_address == 0 {
            self.indirect_errors = std::mem::take(&mut self.errors);
            self.direct_address = self.ops.len();
        }
        (
            self.direct_address,
            Arc::clone(&self.indirect_errors),
            Arc::clone(&self.errors),
        )
    }
    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        if self.direct_address == 0 || op_addr < self.direct_address {
            for (line_number, symbol_addr) in self.symbols.range(0..).rev() {
                if op_addr >= *symbol_addr {
                    return Some(*line_number as u16);
                }
            }
        }
        None
    }
}
