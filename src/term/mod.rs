extern crate ansi_term;
extern crate ctrlc;
extern crate linefeed;
use crate::mach::{Event, Listing, Runtime};
use ansi_term::Style;
use linefeed::{
    Command, Completer, Completion, Function, Interface, Prompter, ReadResult, Signal, Terminal,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn main() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_moved = interrupted.clone();
    ctrlc::set_handler(move || {
        int_moved.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    if let Err(error) = main_loop(interrupted) {
        eprintln!("{}", error);
    }
}

fn main_loop(interrupted: Arc<AtomicBool>) -> std::io::Result<()> {
    let mut runtime = Runtime::default();
    let command = Interface::new("BASIC")?;
    let input_full = Interface::new("Input")?;
    input_full.set_report_signal(Signal::Interrupt, true);
    let input_caps = Interface::new("INPUT")?;
    input_caps.set_report_signal(Signal::Interrupt, true);
    CapsFunction::install(&input_caps);

    loop {
        if interrupted.load(Ordering::SeqCst) {
            runtime.interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match runtime.execute(5000) {
            Event::Stopped => {
                let saved_completer = command.completer();
                command.set_completer(Arc::new(LineCompleter::new(runtime.get_listing())));
                let string = match command.read_line()? {
                    ReadResult::Input(string) => string,
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
                command.set_completer(saved_completer);
                if runtime.enter(&string) {
                    command.add_history_unique(string);
                }
            }
            Event::Input(prompt, caps) => {
                let input = if caps { &input_caps } else { &input_full };
                input.set_prompt(&prompt)?;
                match input.read_line()? {
                    ReadResult::Input(string) => {
                        if runtime.enter(&string) {
                            input.add_history_unique(string);
                        }
                    }
                    ReadResult::Signal(Signal::Interrupt) => {
                        input.set_buffer("")?;
                        // We need the cancel_read_line because ?"Why";:INPUTY
                        // doesn't print the "Why" after you interrupt input.
                        input.lock_reader().cancel_read_line()?;
                        runtime.interrupt();
                    }
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
            }
            Event::Errors(errors) => {
                for error in errors.iter() {
                    let error = format!("{}", error);
                    command.write_fmt(format_args!("{}\n", Style::new().bold().paint(error)))?;
                }
            }
            Event::Running => {}
            Event::Print(s) => {
                command.write_fmt(format_args!("{}", s))?;
            }
            Event::List((s, columns)) => {
                command.write_fmt(format_args!("{}\n", decorate_list(&s, &columns)))?;
            }
            Event::Load(s) => {
                command.write_fmt(format_args!("Loading \"{}\"... (TODO)\n", s))?;
            }
            Event::Save(s) => {
                command.write_fmt(format_args!("Saving \"{}\"... (TODO)\n", s))?;
            }
        }
    }
    Ok(())
}

struct CapsFunction;

impl CapsFunction {
    fn install<T: Terminal>(i: &Interface<T>) {
        i.define_function("caps-function", Arc::new(CapsFunction));
        for ch in 97..=122 {
            i.bind_sequence(
                char::from(ch).to_string(),
                Command::from_str("caps-function"),
            );
        }
    }
}

impl<Term: Terminal> Function<Term> for CapsFunction {
    fn execute(&self, prompter: &mut Prompter<Term>, count: i32, ch: char) -> std::io::Result<()> {
        prompter.insert(count as usize, ch.to_ascii_uppercase())
    }
}

struct LineCompleter {
    runtime: Listing,
}

impl LineCompleter {
    fn new(runtime: Listing) -> LineCompleter {
        LineCompleter { runtime }
    }
}

impl<'a, Term: Terminal> Completer<Term> for LineCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<Term>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        if let Ok(num) = prompter.buffer().parse::<usize>() {
            if let Some((s, _)) = self.runtime.line(num) {
                let mut comp_list = Vec::new();
                let mut comp = Completion::simple(s);
                comp.suffix = linefeed::complete::Suffix::None;
                comp_list.push(comp);
                return Some(comp_list);
            }
        }
        None
    }
}

fn decorate_list(ins: &str, columns: &[std::ops::Range<usize>]) -> String {
    let mut under_on = false;
    let mut out = String::new();
    let style = Style::new().underline();
    let prefix = format!("{}", style.prefix());
    let suffix = format!("{}", style.suffix());
    let mut index = 0;
    for char in ins.chars() {
        let do_under = columns.iter().any(|c| c.contains(&index));
        if under_on {
            if !do_under {
                out.push_str(&suffix);
            }
        } else if do_under {
            out.push_str(&prefix);
        }
        under_on = do_under;
        out.push(char);
        index += 1;
    }
    if columns.iter().any(|c| c.start == index) {
        under_on = true;
        out.push_str(&prefix);
        out.push(' ');
    }
    if under_on {
        out.push_str(&suffix);
    }
    out
}
