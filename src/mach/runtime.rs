use super::{Address, Op, Program, Stack, Val};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber};
use std::collections::{BTreeMap, HashMap};
type Result<T> = std::result::Result<T, Error>;
use std::rc::Rc;

/// ## Virtual machine

pub struct Runtime {
    source: BTreeMap<LineNumber, Line>,
    dirty: bool,
    program: Program,
    pc: Address,
    direct: Address,
    indirect_errors: Rc<Vec<Error>>,
    direct_errors: Rc<Vec<Error>>,
    errors: Rc<Vec<Error>>,
    stack: Stack<Val>,
    vars: HashMap<String, Val>,
    state: Status,
}

/// ## Events for the user interface

pub enum Event {
    Errors(Rc<Vec<Error>>),
    PrintLn(String),
    Stopped,
    Running,
}

#[derive(PartialEq)]
enum Status {
    Intro,
    Stopped,
    Running,
    DirectErrors,
    Interrupt,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            source: BTreeMap::new(),
            dirty: false,
            program: Program::new(),
            pc: 0,
            direct: 0,
            indirect_errors: Rc::new(vec![]),
            direct_errors: Rc::new(vec![]),
            errors: Rc::new(vec![]),
            stack: Stack::new("STACK OVERFLOW"),
            vars: HashMap::new(),
            state: Status::Intro,
        }
    }

    pub fn lines<T: std::ops::RangeBounds<LineNumber>>(
        &self,
        range: T,
    ) -> Vec<(String, Vec<Column>)> {
        let mut r: Vec<(String, Vec<Column>)> = vec![];
        for (line_number, line) in self.source.range(range) {
            let iter = self
                .indirect_errors
                .iter()
                .chain(self.direct_errors.iter().chain(self.errors.iter()));
            let mut columns: Vec<Column> = iter
                .filter_map(|e| {
                    if e.line_number() == *line_number {
                        Some(e.column())
                    } else {
                        None
                    }
                })
                .collect();
            match line_number {
                Some(num) => {
                    let offset = num.to_string().len() + 1;
                    for column in &mut columns {
                        *column = (column.start + offset)..(column.end + offset);
                    }
                }
                None => {}
            };
            r.push((line.to_string(), columns));
        }
        r
    }

    pub fn enter(&mut self, s: &str) {
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
            self.direct = pc;
            self.indirect_errors = indirect_errors;
            self.direct_errors = direct_errors;
            self.errors = Rc::new(vec![]);
            if self.direct_errors.is_empty() {
                self.state = Status::Running;
            } else {
                self.state = Status::DirectErrors;
            }
        } else {
            self.source.insert(line.number(), line);
            self.stack.clear();
            self.dirty = true;
        };
    }

    pub fn is_stopped(&self) -> bool {
        self.state == Status::Stopped
    }

    pub fn interrupt(&mut self) {
        self.state = Status::Interrupt;
    }

    pub fn execute(&mut self, iterations: usize) -> Event {
        fn adj(pc: Address) -> Address {
            if pc > 0 {
                pc - 1
            } else {
                pc
            }
        }
        match self.state {
            Status::Intro => {
                self.state = Status::Stopped;
                return Event::PrintLn("64K BASIC SYSTEM".to_string());
            }
            Status::Stopped => return Event::Stopped,
            Status::Running => {}
            Status::DirectErrors => {
                self.state = Status::Stopped;
                return Event::Errors(Rc::clone(&self.direct_errors));
            }
            Status::Interrupt => {
                self.state = Status::Stopped;
                let line_number = self.program.line_number_for(adj(self.pc));
                return Event::Errors(Rc::new(vec![error!(Break, line_number)]));
            }
        }
        match self.execute_loop(iterations) {
            Ok(e) => e,
            Err(e) => {
                self.state = Status::Stopped;
                let line_number = self.program.line_number_for(adj(self.pc));
                Rc::make_mut(&mut self.errors).push(e.in_line_number(line_number));
                Event::Errors(Rc::clone(&self.errors))
            }
        }
    }

    fn execute_loop(&mut self, iterations: usize) -> Result<Event> {
        let has_indirect_errors = !self.indirect_errors.is_empty();
        for _ in 0..iterations {
            let op = match self.program.ops().get(self.pc) {
                Some(v) => v,
                None => return Err(error!(InternalError; "INVALID PC ADDRESS")),
            };
            self.pc += 1;
            match op {
                Op::Literal(val) => self.stack.push(val.clone())?,
                Op::Push(var_name) => {
                    self.stack.push(match self.vars.get(var_name) {
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
                    })?;
                }
                Op::Run => {
                    if has_indirect_errors {
                        self.state = Status::Stopped;
                        return Ok(Event::Errors(Rc::clone(&self.indirect_errors)));
                    }
                    self.stack.clear();
                    self.vars.clear();
                    self.pc = 0;
                }
                Op::Jump(addr) => {
                    self.pc = *addr;
                    if has_indirect_errors && self.pc < self.direct {
                        self.state = Status::Stopped;
                        return Ok(Event::Errors(Rc::clone(&self.indirect_errors)));
                    }
                }
                Op::End => {
                    self.pc -= 1;
                    self.state = Status::Stopped;
                    return Ok(Event::Stopped);
                }
                Op::Neg => self.r#neg()?,
                Op::Add => self.r#add()?,
                Op::List => {
                    dbg!(self.stack.pop()?);
                    dbg!(self.stack.pop()?);
                }
                Op::Print => self.r#print()?,
                _ => {
                    dbg!(&op);
                    return Err(error!(InternalError; "OP NOT YET RUNNING; PANIC"));
                }
            }
        }
        Ok(Event::Running)
    }

    fn r#neg(&mut self) -> Result<()> {
        let val = self.stack.pop()?;
        self.stack.push(Val::neg(val)?)?;
        Ok(())
    }

    fn r#add(&mut self) -> Result<()> {
        let (lhs, rhs) = self.stack.pop_2()?;
        self.stack.push(Val::add(lhs, rhs)?)?;
        Ok(())
    }

    fn r#print(&mut self) -> Result<()> {
        match self.stack.pop()? {
            Val::Integer(len) => {
                for item in self.stack.pop_n(len as usize)? {
                    print!("{}", item);
                }
                Ok(())
            }
            _ => return Err(error!(InternalError; "EXPECTED VECTOR ON STACK")),
        }
    }
}
