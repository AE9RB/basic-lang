use super::{Address, Op, Program, Stack, Val, Var};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber, MaxValue};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ops::Range;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

const INTRO: &str = "64K BASIC";
const READY: &str = "READY.";

/// ## Virtual machine

pub struct Runtime {
    source: BTreeMap<LineNumber, Line>,
    dirty: bool,
    program: Program,
    pc: Address,
    entry_address: Address,
    indirect_errors: Arc<Vec<Error>>,
    direct_errors: Arc<Vec<Error>>,
    stack: Stack<Val>,
    vars: Var,
    state: Status,
}

/// ## Events for the user interface

pub enum Event {
    Errors(Arc<Vec<Error>>),
    Print(String),
    List((String, Vec<Range<usize>>)),
    Stopped,
    Running,
}

#[derive(Debug, PartialEq)]
enum Status {
    Intro,
    Stopped,
    Listing(Range<LineNumber>),
    Running,
    Interrupt,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            source: BTreeMap::new(),
            dirty: false,
            program: Program::new(),
            pc: 0,
            entry_address: 1,
            indirect_errors: Arc::new(vec![]),
            direct_errors: Arc::new(vec![]),
            stack: Stack::new("STACK OVERFLOW"),
            vars: Var::new(),
            state: Status::Intro,
        }
    }
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime::default()
    }

    /// Enters a line of BASIC.
    /// Returns true if it was a non-blank direct line.
    /// Return value is useful for command history.
    pub fn enter(&mut self, s: &str) -> bool {
        if s.trim().is_empty() {
            return false;
        }
        let line = Line::new(s);
        if line.is_direct() {
            if self.dirty {
                self.program.clear();
                self.program
                    .compile(self.source.iter().map(|(_, line)| line));
                self.dirty = false;
            }
            self.program.compile(&line);
            let (pc, indirect_errors, direct_errors) = self.program.link();
            self.pc = pc;
            self.entry_address = pc;
            self.indirect_errors = indirect_errors;
            self.direct_errors = direct_errors;
            self.state = Status::Running;
            true
        } else {
            if line.is_empty() {
                self.dirty = self.source.remove(&line.number()).is_some();
            } else {
                self.source.insert(line.number(), line);
                self.dirty = true;
            }
            false
        }
    }

    pub fn interrupt(&mut self) {
        self.state = Status::Interrupt;
    }

    pub fn line(&self, num: usize) -> Option<(String, Vec<Range<usize>>)> {
        if num > LineNumber::max_value() as usize {
            return None;
        }
        let mut range = Some(num as u16)..Some(num as u16);
        self.list_line(&mut range)
    }

    fn list_line(&self, range: &mut Range<LineNumber>) -> Option<(String, Vec<Range<usize>>)> {
        let mut source_range = self.source.range(range.start..=range.end);
        if let Some((line_number, line)) = source_range.next() {
            if *line_number < range.end {
                if let Some(num) = line_number {
                    range.start = Some(num + 1);
                }
            } else {
                range.start = Some(0);
                range.end = Some(0);
            }
            let columns: Vec<Column> = self
                .indirect_errors
                .iter()
                .filter_map(|e| {
                    if e.line_number() == *line_number {
                        Some(e.column())
                    } else {
                        None
                    }
                })
                .collect();
            return Some((line.to_string(), columns));
        }
        None
    }

    pub fn execute(&mut self, iterations: usize) -> Event {
        fn prev_pc(pc: Address) -> Address {
            if pc > 0 {
                pc - 1
            } else {
                pc
            }
        }
        match &self.state {
            Status::Intro => {
                self.state = Status::Stopped;
                let mut s = INTRO.to_string();
                if let Some(version) = option_env!("CARGO_PKG_VERSION") {
                    s.push(' ');
                    s.push_str(version);
                }
                #[cfg(debug_assertions)]
                s.push_str("+debug");
                s.push('\n');
                return Event::Print(s);
            }
            Status::Stopped => {
                if self.entry_address != 0 {
                    self.entry_address = 0;
                    let mut s = READY.to_string();
                    s.push('\n');
                    return Event::Print(s);
                }
                return Event::Stopped;
            }
            Status::Interrupt => {
                self.state = Status::Stopped;
                let line_number = self.program.line_number_for(prev_pc(self.pc));
                return Event::Errors(Arc::new(vec![error!(Break, line_number)]));
            }
            Status::Listing(range) => {
                let mut range = range.clone();
                if let Some((string, columns)) = self.list_line(&mut range) {
                    self.state = Status::Listing(range);
                    return Event::List((string, columns));
                } else {
                    self.state = Status::Running;
                }
            }
            Status::Running => {
                if !self.direct_errors.is_empty() {
                    self.state = Status::Stopped;
                    return Event::Errors(Arc::clone(&self.direct_errors));
                }
            }
        }
        debug_assert_eq!(self.state, Status::Running);
        match self.execute_loop(iterations) {
            Ok(event) => {
                if self.state == Status::Stopped {
                    match event {
                        Event::Stopped => {
                            self.entry_address = 0;
                            let mut s = READY.to_string();
                            s.push('\n');
                            Event::Print(s)
                        }
                        _ => event,
                    }
                } else {
                    event
                }
            }
            Err(error) => {
                self.state = Status::Stopped;
                let line_number = self.program.line_number_for(prev_pc(self.pc));
                Event::Errors(Arc::new(vec![error.in_line_number(line_number)]))
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn execute_loop(&mut self, iterations: usize) -> Result<Event> {
        let has_indirect_errors = !self.indirect_errors.is_empty();
        for _ in 0..iterations {
            let op = match self.program.get(self.pc) {
                Some(v) => v,
                None => return Err(error!(InternalError; "INVALID PC ADDRESS")),
            };
            self.pc += 1;
            match op {
                Op::For(addr) => {
                    panic!("{}", &addr);
                }
                Op::Literal(val) => self.stack.push(val.clone())?,
                Op::Pop(var_name) => self.vars.store(var_name, self.stack.pop()?)?,
                Op::Push(var_name) => self.stack.push(self.vars.fetch(var_name))?,
                Op::If(_) => return Err(error!(InternalError; "'IF' NOT YET IMPLEMENTED; PANIC")),
                Op::Jump(addr) => {
                    self.pc = *addr;
                    if has_indirect_errors && self.pc < self.entry_address {
                        self.state = Status::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.indirect_errors)));
                    }
                }
                Op::Return => {
                    return Err(error!(InternalError; "'RETURN' NOT YET IMPLEMENTED; PANIC"))
                }
                Op::Run => {
                    if has_indirect_errors {
                        self.state = Status::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.indirect_errors)));
                    }
                    self.stack.clear();
                    self.vars.clear();
                    self.pc = 0;
                }

                Op::List => return self.r#list(),
                Op::End => return self.r#end(),
                Op::Print => return self.r#print(),

                Op::Neg => self.r#negation()?,
                Op::Exp => self.pop_2_op(&Val::unimplemented)?,
                Op::Mul => self.pop_2_op(&Val::multiply)?,
                Op::Div => self.pop_2_op(&Val::divide)?,
                Op::DivInt => self.pop_2_op(&Val::unimplemented)?,
                Op::Mod => self.pop_2_op(&Val::unimplemented)?,
                Op::Add => self.pop_2_op(&Val::sum)?,
                Op::Sub => self.pop_2_op(&Val::subtract)?,
                Op::Eq => self.pop_2_op(&Val::unimplemented)?,
                Op::NotEq => self.pop_2_op(&Val::unimplemented)?,
                Op::Lt => self.pop_2_op(&Val::unimplemented)?,
                Op::LtEq => self.pop_2_op(&Val::unimplemented)?,
                Op::Gt => self.pop_2_op(&Val::unimplemented)?,
                Op::GtEq => self.pop_2_op(&Val::unimplemented)?,
                Op::Not => self.pop_2_op(&Val::unimplemented)?,
                Op::And => self.pop_2_op(&Val::unimplemented)?,
                Op::Or => self.pop_2_op(&Val::unimplemented)?,
                Op::Xor => self.pop_2_op(&Val::unimplemented)?,
                Op::Imp => self.pop_2_op(&Val::unimplemented)?,
                Op::Eqv => self.pop_2_op(&Val::unimplemented)?,
            }
        }
        Ok(Event::Running)
    }

    fn pop_2_op<T: Fn(Val, Val) -> Result<Val>>(&mut self, func: &T) -> Result<()> {
        let (lhs, rhs) = self.stack.pop_2()?;
        self.stack.push(func(lhs, rhs)?)?;
        Ok(())
    }

    fn r#negation(&mut self) -> Result<()> {
        let val = self.stack.pop()?;
        self.stack.push(Val::negate(val)?)?;
        Ok(())
    }

    fn r#list(&mut self) -> Result<Event> {
        let (from, to) = self.stack.pop_2()?;
        let from = LineNumber::try_from(from)?;
        let to = LineNumber::try_from(to)?;
        if to < from {
            return Err(error!(UndefinedLine; "INVALID RANGE"));
        }
        self.state = Status::Listing(from..to);
        Ok(Event::Running)
    }

    fn r#end(&mut self) -> Result<Event> {
        self.pc -= 1;
        self.state = Status::Stopped;
        Ok(Event::Stopped)
    }

    fn r#print(&mut self) -> Result<Event> {
        match self.stack.pop()? {
            Val::Integer(len) => {
                let mut s = String::new();
                for item in self.stack.pop_n(len as usize)? {
                    s.push_str(&format!("{}", item));
                }
                Ok(Event::Print(s))
            }
            _ => Err(error!(InternalError; "EXPECTED VECTOR ON STACK")),
        }
    }
}
