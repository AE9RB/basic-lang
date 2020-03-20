use super::{Address, Function, Opcode, Operation, Program, Stack, Val, Var};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber, MaxValue};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ops::Range;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

const INTRO: &str = "64K BASIC";
const MAX_LINE_LEN: usize = 1024;

/// ## Virtual machine

pub struct Runtime {
    prompt: String,
    source: BTreeMap<LineNumber, Line>,
    dirty: bool,
    program: Program,
    pc: Address,
    entry_address: Address,
    indirect_errors: Arc<Vec<Error>>,
    direct_errors: Arc<Vec<Error>>,
    stack: RuntimeStack,
    vars: Var,
    state: State,
    cont: State,
    cont_pc: Address,
    print_col: usize,
}

/// ## Events for the user interface

pub enum Event {
    Errors(Arc<Vec<Error>>),
    Input(String, bool),
    Print(String),
    List((String, Vec<Range<usize>>)),
    Running,
    Stopped,
}

#[derive(Debug)]
enum State {
    Intro,
    Stopped,
    Listing(Range<LineNumber>),
    RuntimeError(Error),
    Running,
    Input(bool),
    InputRedo(bool),
    Interrupt,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            prompt: "READY.".to_owned(),
            source: BTreeMap::new(),
            dirty: false,
            program: Program::new(),
            pc: 0,
            entry_address: 1,
            indirect_errors: Arc::new(vec![]),
            direct_errors: Arc::new(vec![]),
            stack: Stack::new("STACK OVERFLOW"),
            vars: Var::new(),
            state: State::Intro,
            cont: State::Stopped,
            cont_pc: 0,
            print_col: 0,
        }
    }
}

impl Runtime {
    pub fn new(prompt: &str) -> Runtime {
        let mut rt = Runtime::default();
        rt.prompt = prompt.to_owned();
        rt
    }

    /// Enters a line of BASIC or INPUT.
    /// Returns true if good candidate for history.
    pub fn enter(&mut self, string: &str) -> bool {
        if let State::Input(_) = self.state {
            self.enter_input(string);
            self.print_col = 0;
            return true;
        }
        debug_assert!(matches!(self.state, State::Stopped | State::Intro));
        if string.len() > MAX_LINE_LEN {
            self.state = State::RuntimeError(error!(OutOfMemory));
            return false;
        }
        let line = Line::new(string);
        if line.is_direct() {
            if line.is_empty() {
                false
            } else {
                self.enter_direct(line);
                true
            }
        } else {
            self.enter_indirect(line);
            false
        }
    }

    fn enter_direct(&mut self, line: Line) {
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
        self.state = State::Running;
    }

    fn enter_indirect(&mut self, line: Line) {
        self.cont = State::Stopped;
        self.stack.clear();
        if line.is_empty() {
            self.dirty = self.source.remove(&line.number()).is_some();
        } else {
            self.source.insert(line.number(), line);
            self.dirty = true;
        }
    }

    fn enter_input(&mut self, string: &str) {
        if string.len() <= MAX_LINE_LEN && self.apply_input(string).is_ok() {
            self.state = State::Running;
            return;
        }
        if let State::Input(true) = self.state {
            self.state = State::InputRedo(true);
        } else {
            self.state = State::InputRedo(false);
        }
    }

    fn apply_input(&mut self, string: &str) -> Result<()> {
        let prompt = self.stack.pop()?;
        if let Val::Integer(len) = self.stack.pop()? {
            let mut vals = self.parse_input(string);
            if vals.len() != len as usize {
                self.stack.push(Val::Integer(len))?;
                self.stack.push(prompt)?;
                return Err(error!(SyntaxError));
            }
            let var_names = self.stack.pop_n(len as usize)?;
            let mut old_vals: Vec<Val> = vec![];
            let mut redo = false;
            for (name, (s, val)) in var_names.iter().zip(vals.drain(..)) {
                if let Val::String(name_str) = name {
                    old_vals.push(self.vars.fetch(name_str));
                    if s.is_empty() {
                        self.vars.remove(&name_str);
                        continue;
                    }
                    if self.vars.store(name_str, val).is_ok() {
                        continue;
                    }
                    if Var::is_string(&name_str) {
                        self.vars.store(&name_str, Val::String(s.to_string()))?;
                        continue;
                    }
                }
                redo = true;
                break;
            }
            if redo {
                for (name, val) in var_names.iter().zip(old_vals.drain(..)) {
                    if let Val::String(name_str) = name {
                        self.vars.store(name_str, val).ok();
                    }
                }
                for val in var_names {
                    self.stack.push(val)?;
                }
                self.stack.push(Val::Integer(len))?;
                self.stack.push(prompt)?;
                return Err(error!(SyntaxError));
            }
        }
        Ok(())
    }

    fn parse_input<'a>(&mut self, string: &'a str) -> Vec<(&'a str, Val)> {
        let mut v: Vec<(&str, Val)> = vec![];
        let mut start: usize = 0;
        let mut in_quote = false;
        for (i, ch) in string.char_indices() {
            match ch {
                '"' => {
                    in_quote = !in_quote;
                }
                ',' if !in_quote => {
                    v.push((&string[start..i].trim(), Val::from(&string[start..i])));
                    start = i + 1;
                }
                _ => {}
            }
        }
        v.push((&string[start..].trim(), Val::from(&string[start..])));
        v
    }

    pub fn interrupt(&mut self) {
        self.cont = State::Interrupt;
        std::mem::swap(&mut self.state, &mut self.cont);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
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

    fn ready_prompt(&mut self) -> Option<Event> {
        if self.entry_address != 0 {
            self.entry_address = 0;
            let mut s = String::new();
            if self.print_col > 0 {
                s.push('\n');
                self.print_col = 0;
            }
            s.push_str(&self.prompt);
            s.push('\n');
            return Some(Event::Print(s));
        };
        None
    }

    pub fn execute(&mut self, iterations: usize) -> Event {
        fn line_number(this: &Runtime) -> LineNumber {
            let mut pc = this.pc;
            if pc > 0 {
                pc -= 1
            }
            this.program.line_number_for(pc)
        }
        match &self.state {
            State::Intro => {
                self.state = State::Stopped;
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
            State::Stopped => match self.ready_prompt() {
                Some(e) => return e,
                None => return Event::Stopped,
            },
            State::Interrupt => {
                self.state = State::RuntimeError(error!(Break, line_number(&self)));
            }
            State::Listing(range) => {
                let mut range = range.clone();
                if let Some((string, columns)) = self.list_line(&mut range) {
                    self.state = State::Listing(range);
                    return Event::List((string, columns));
                } else {
                    self.state = State::Running;
                }
            }
            State::Input(caps) => {
                if let Some(Val::String(prompt)) = self.stack.last() {
                    self.print_col += prompt.chars().count();
                    return Event::Input(prompt.clone(), *caps);
                }
                self.state = State::RuntimeError(
                    error!(InternalError, line_number(&self); "NO INPUT PROMPT ON STACK"),
                );
            }
            State::InputRedo(caps) => {
                self.state = State::Input(*caps);
                return Event::Errors(Arc::new(vec![error!(RedoFromStart)]));
            }
            State::Running => {
                if !self.direct_errors.is_empty() {
                    self.state = State::Stopped;
                    return Event::Errors(Arc::clone(&self.direct_errors));
                }
            }
            State::RuntimeError(_) => {}
        }
        if let State::RuntimeError(_) = self.state {
            if self.print_col > 0 {
                self.print_col = 0;
                return Event::Print('\n'.to_string());
            }
            let mut state = State::Stopped;
            std::mem::swap(&mut self.state, &mut state);
            if let State::RuntimeError(e) = state {
                return Event::Errors(Arc::new(vec![e]));
            }
        }
        debug_assert!(matches!(self.state, State::Running));
        match self.execute_loop(iterations) {
            Ok(event) => {
                if let State::Stopped = self.state {
                    match event {
                        Event::Stopped => match self.ready_prompt() {
                            Some(e) => e,
                            None => event,
                        },
                        _ => event,
                    }
                } else {
                    event
                }
            }
            Err(error) => {
                self.cont = State::RuntimeError(error.in_line_number(line_number(&self)));
                std::mem::swap(&mut self.cont, &mut self.state);
                self.cont_pc = self.pc;
                if self.pc >= self.entry_address || self.stack.is_full() {
                    self.stack.clear();
                    self.cont = State::Stopped;
                }
                Event::Running
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
                Opcode::Literal(val) => self.stack.push(val.clone())?,
                Opcode::Pop(var_name) => self.vars.store(var_name, self.stack.pop()?)?,
                Opcode::Push(var_name) => self.stack.push(self.vars.fetch(var_name))?,
                Opcode::PopArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.stack.pop()?;
                    self.vars.store_array(var_name, vec, val)?;
                }
                Opcode::PushArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.vars.fetch_array(var_name, vec)?;
                    self.stack.push(val)?;
                }
                Opcode::For(addr) => {
                    let addr = *addr;
                    self.r#for(addr)?;
                }
                Opcode::If(_) => {
                    return Err(error!(InternalError; "'IF' NOT YET IMPLEMENTED; PANIC"))
                }
                Opcode::Jump(addr) => {
                    self.pc = *addr;
                    if has_indirect_errors && self.pc < self.entry_address {
                        self.state = State::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.indirect_errors)));
                    }
                }
                Opcode::Return => {
                    return Err(error!(InternalError; "'RETURN' NOT YET IMPLEMENTED; PANIC"))
                }
                Opcode::Clear => {
                    self.stack.clear();
                    self.vars.clear();
                }
                Opcode::Cont => {
                    if let State::Stopped = self.cont {
                        return Err(error!(CantContinue));
                    }
                    if let State::Running = self.state {
                        self.state = State::Stopped;
                        std::mem::swap(&mut self.cont, &mut self.state);
                        self.pc = self.cont_pc;
                    } else {
                        return Err(error!(CantContinue));
                    }
                    if let State::Running = self.state {
                    } else {
                        return Ok(Event::Running);
                    }
                }
                Opcode::Input => return self.r#input(),
                Opcode::List => return self.r#list(),
                Opcode::End => return self.r#end(),
                Opcode::Print => return self.r#print(),
                Opcode::Stop => {
                    self.r#end()?;
                    return Err(error!(Break));
                }

                Opcode::Neg => self.stack.pop_1_push(&Operation::negate)?,
                Opcode::Exp => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Mul => self.stack.pop_2_push(&Operation::multiply)?,
                Opcode::Div => self.stack.pop_2_push(&Operation::divide)?,
                Opcode::DivInt => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Mod => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Add => self.stack.pop_2_push(&Operation::sum)?,
                Opcode::Sub => self.stack.pop_2_push(&Operation::subtract)?,
                Opcode::Eq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::NotEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Lt => self.stack.pop_2_push(&Operation::less)?,
                Opcode::LtEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Gt => self.stack.pop_2_push(&Operation::greater)?,
                Opcode::GtEq => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Not => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::And => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Or => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Xor => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Imp => self.stack.pop_2_push(&Operation::unimplemented)?,
                Opcode::Eqv => self.stack.pop_2_push(&Operation::unimplemented)?,

                Opcode::Cos => self.stack.pop_1_push(&Function::cos)?,
                Opcode::Sin => self.stack.pop_1_push(&Function::sin)?,
            }
        }
        Ok(Event::Running)
    }

    fn r#for(&mut self, addr: Address) -> Result<()> {
        loop {
            if self.stack.len() < 4 {
                break;
            }
            let (first_iter, next_name) = match self.stack.pop()? {
                Val::String(s) => (false, s),
                _ => (true, "".to_string()),
            };
            let var_name_val = self.stack.pop()?;
            let to_val = self.stack.pop()?;
            let step_val = self.stack.pop()?;
            if let Val::String(var_name) = var_name_val {
                if !next_name.is_empty() && var_name != next_name {
                    self.stack.push(Val::String(next_name))?;
                    continue;
                }
                let mut current = self.vars.fetch(&var_name);
                if !first_iter {
                    current = Operation::sum(current, step_val.clone())?;
                    self.vars.store(&var_name, current.clone())?;
                }
                if let Ok(step) = f64::try_from(step_val.clone()) {
                    let done = Val::Integer(-1)
                        == if step < 0.0 {
                            Operation::less(current, to_val.clone())?
                        } else {
                            Operation::less(to_val.clone(), current)?
                        };
                    if done {
                        self.pc = addr;
                    } else {
                        self.stack.push(step_val)?;
                        self.stack.push(to_val)?;
                        self.stack.push(Val::String(var_name))?;
                    }
                    return Ok(());
                }
            }
            break;
        }
        Err(error!(NextWithoutFor; "MISSING STACK FRAME"))
    }

    fn r#input(&mut self) -> Result<Event> {
        if let Val::Integer(i) = self.stack.pop()? {
            let caps = i != 0;
            if let Val::String(mut s) = self.stack.pop()? {
                s.push('?');
                s.push(' ');
                let prompt = s.clone();
                self.stack.push(Val::String(s))?;
                self.state = State::Input(caps);
                self.print_col += prompt.chars().count();
                return Ok(Event::Input(prompt, caps));
            }
        }
        Err(error!(InternalError))
    }

    fn r#list(&mut self) -> Result<Event> {
        let (from, to) = self.stack.pop_2()?;
        let from = LineNumber::try_from(from)?;
        let to = LineNumber::try_from(to)?;
        if to < from {
            return Err(error!(UndefinedLine; "INVALID RANGE"));
        }
        self.state = State::Listing(from..to);
        Ok(Event::Running)
    }

    fn r#end(&mut self) -> Result<Event> {
        self.cont = State::Stopped;
        std::mem::swap(&mut self.cont, &mut self.state);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
        Ok(Event::Stopped)
    }

    fn r#print(&mut self) -> Result<Event> {
        let mut s = String::new();
        for item in self.stack.pop_vec()? {
            match item {
                Val::Char('\n') => {
                    s.push('\n');
                    self.print_col = 0;
                }
                Val::Char('\t') => {
                    let len = 14 - (self.print_col % 14);
                    s.push_str(&" ".repeat(len));
                    self.print_col += len;
                }
                _ => {
                    let val_str = format!("{}", item);
                    for ch in val_str.chars() {
                        s.push(ch);
                        match ch {
                            '\n' => self.print_col = 0,
                            _ => self.print_col += 1,
                        }
                    }
                }
            }
        }
        Ok(Event::Print(s))
    }
}

type RuntimeStack = Stack<Val>;

trait RuntimeStackTrait<T> {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_vec(&mut self) -> Result<Vec<Val>>;
}

impl RuntimeStackTrait<Val> for RuntimeStack {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let val = self.pop()?;
        self.push(func(val)?)?;
        Ok(())
    }
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let (val1, val2) = self.pop_2()?;
        self.push(func(val1, val2)?)?;
        Ok(())
    }
    fn pop_vec(&mut self) -> Result<Vec<Val>> {
        if let Val::Integer(n) = self.pop()? {
            self.pop_n(n as usize)
        } else {
            Err(error!(InternalError; "NO VECTOR ON STACK"))
        }
    }
}
