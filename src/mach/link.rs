use super::{Address, Opcode, Stack, Symbol, Val};
use crate::error;
use crate::lang::{Column, Error, LineNumber, MaxValue};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

/// ## Linkable object

#[derive(Debug)]
pub struct Link {
    shared: Rc<RefCell<LinkShared>>,
    ops: Stack<Opcode>,
    symbols: BTreeMap<Symbol, Address>,
    unlinked: HashMap<Address, (Column, Symbol)>,
}

#[derive(Debug, Default)]
struct LinkShared {
    current_symbol: Symbol,
    loops: Vec<(Column, String, Symbol, Symbol)>,
}

impl Default for Link {
    fn default() -> Self {
        Link {
            shared: Rc::default(),
            ops: Stack::new("PROGRAM TOO LARGE"),
            symbols: BTreeMap::default(),
            unlinked: HashMap::default(),
        }
    }
}

impl Link {
    pub fn new(&mut self) -> Link {
        let mut link = Link::default();
        link.shared = Rc::clone(&self.shared);
        link
    }

    pub fn append(&mut self, mut link: Link) -> Result<()> {
        debug_assert!(Rc::ptr_eq(&self.shared, &link.shared));
        let offset = self.ops.len();
        for (symbol, address) in link.symbols.iter() {
            self.symbols.insert(*symbol, *address + offset);
        }
        for (address, cs) in link.unlinked.iter() {
            self.unlinked.insert(*address + offset, cs.clone());
        }
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
        self.shared.borrow_mut().current_symbol = 0;
        self.shared.borrow_mut().loops.clear();
        self.ops.clear();
        self.symbols.clear();
        self.unlinked.clear();
    }

    pub fn next_symbol(&mut self) -> Symbol {
        self.shared.borrow_mut().current_symbol -= 1;
        self.shared.borrow().current_symbol
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

    fn begin_for_loop(&mut self, addr: Address, col: Column, var_name: String) -> Result<()> {
        let loop_start = self.next_symbol();
        let loop_end = self.next_symbol();
        self.shared
            .borrow_mut()
            .loops
            .push((col.clone(), var_name, loop_start, loop_end));
        self.insert(loop_start, addr);
        self.link_addr_to_symbol(addr, col, loop_end);
        Ok(())
    }

    pub fn next_for_loop(&mut self, addr: Address, col: Column, var_name: String) -> Result<()> {
        let (_col, _for_name, loop_start, loop_end) = match self.shared.borrow_mut().loops.pop() {
            Some((col, for_name, loop_start, loop_end)) => {
                if var_name.is_empty() || var_name == for_name {
                    (col, for_name, loop_start, loop_end)
                } else {
                    return Err(error!(NextWithoutFor, ..&col));
                }
            }
            _ => return Err(error!(NextWithoutFor, ..&col)),
        };

        self.link_addr_to_symbol(addr, col, loop_start);
        self.insert(loop_end, addr + 1);
        Ok(())
    }

    pub fn push_for(&mut self, col: Column, ident: String) -> Result<()> {
        self.begin_for_loop(self.ops.len(), col, ident)?;
        self.ops.push(Opcode::For(0))
    }

    pub fn push_next(&mut self, col: Column, ident: String) -> Result<()> {
        self.ops.push(Opcode::Literal(Val::String(ident.clone())))?;
        self.next_for_loop(self.ops.len(), col, ident)?;
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_goto(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        let sym = self.symbol_for_line_number(line_number)?;
        self.link_addr_to_symbol(self.ops.len(), col, sym);
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_run(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        self.ops.push(Opcode::Clear)?;
        if line_number.is_some() {
            let sym = self.symbol_for_line_number(line_number)?;
            self.link_addr_to_symbol(self.ops.len(), col, sym);
        }
        self.ops.push(Opcode::Jump(0))
    }

    pub fn set_start_of_direct(&mut self, op_addr: Address) {
        self.insert(LineNumber::max_value() as isize + 1 as Symbol, op_addr);
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
        for (col, _, loop_start, _) in self.shared.borrow_mut().loops.drain(..) {
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
                    if let Some(op) = self.ops.get_mut(op_addr) {
                        if let Some(new_op) = match op {
                            Opcode::For(_) => Some(Opcode::For(*dest)),
                            Opcode::IfNot(_) => Some(Opcode::IfNot(*dest)),
                            Opcode::Jump(_) => Some(Opcode::Jump(*dest)),
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
        self.shared.borrow_mut().current_symbol = 0;
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
