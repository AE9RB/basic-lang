extern crate ansi_term;
extern crate crc;
extern crate ctrlc;
extern crate linefeed;
extern crate mortal;
extern crate reqwest;
use crate::mach::{Event, Listing, Runtime};
use crate::{error, lang::Error};
use ansi_term::Style;
use crc::Hasher32;
use linefeed::{
    Command, Completer, Completion, Function, Interface, Prompter, ReadResult, Signal, Terminal,
};
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn main() {
    if std::env::args().count() > 2 {
        println!("Usage: basic [FILENAME]");
        return;
    }
    let mut args = std::env::args();
    let _executable = args.next();
    let filename = match args.next() {
        Some(f) => f,
        _ => "".into(),
    };
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_moved = interrupted.clone();
    ctrlc::set_handler(move || {
        int_moved.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    if let Err(error) = main_loop(interrupted, filename) {
        eprintln!("{}", error);
    }
}

fn main_loop(interrupted: Arc<AtomicBool>, filename: String) -> std::io::Result<()> {
    let terminal = mortal::Terminal::new()?;
    let mut runtime = Runtime::default();
    let command = Interface::new("BASIC")?;
    let input_full = Interface::new("Input")?;
    input_full.set_report_signal(Signal::Interrupt, true);
    let input_caps = Interface::new("INPUT")?;
    input_caps.set_report_signal(Signal::Interrupt, true);
    CapsFunction::install(&input_caps);

    if !filename.is_empty() {
        match load(&filename, true, false) {
            Ok(listing) => {
                if listing.is_empty() {
                    return Ok(());
                }
                runtime.set_prompt("");
                runtime.set_listing(listing, true);
            }
            Err(error) => {
                command.write_fmt(format_args!(
                    "{}\n",
                    Style::new().bold().paint(error.to_string())
                ))?;
                return Ok(());
            }
        }
    }

    loop {
        if interrupted.load(Ordering::SeqCst) {
            runtime.interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match runtime.execute(5000) {
            Event::Stopped => {
                if !filename.is_empty() {
                    return Ok(());
                }
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
                        // We need the cancel_read_line because ?"Why";:INPUT Y
                        // doesn't print the "Why" after you interrupt input.
                        input.lock_reader().cancel_read_line()?;
                        runtime.interrupt();
                    }
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
            }
            Event::Errors(errors) => {
                for error in errors.iter() {
                    command.write_fmt(format_args!(
                        "{}\n",
                        Style::new().bold().paint(error.to_string())
                    ))?;
                }
            }
            Event::Running => {}
            Event::Print(s) => {
                command.write_fmt(format_args!("{}", s))?;
            }
            Event::List((s, columns)) => {
                command.write_fmt(format_args!("{}\n", decorate_list(&s, &columns)))?;
            }
            Event::Load(s) => match load(&s, false, false) {
                Ok(listing) => runtime.set_listing(listing, false),
                Err(error) => command.write_fmt(format_args!(
                    "{}\n",
                    Style::new().bold().paint(error.to_string())
                ))?,
            },
            Event::Run(s) => match load(&s, false, false) {
                Ok(listing) => runtime.set_listing(listing, true),
                Err(error) => command.write_fmt(format_args!(
                    "{}\n",
                    Style::new().bold().paint(error.to_string())
                ))?,
            },
            Event::Save(s) => match save(&runtime.get_listing(), &s) {
                Ok(_) => {}
                Err(error) => command.write_fmt(format_args!(
                    "{}\n",
                    Style::new().bold().paint(error.to_string())
                ))?,
            },
            Event::Cls => {
                terminal.clear_screen()?;
            }
            Event::Inkey => {
                let mut s: std::rc::Rc<str> = "".into();
                loop {
                    match terminal.read_event(Some(std::time::Duration::from_millis(1)))? {
                        Some(mortal::terminal::Event::Key(key)) => {
                            use mortal::terminal::Key::*;
                            s = match key {
                                Backspace => "\x08".into(),
                                Enter => "\x0D".into(),
                                Escape => "\x1B".into(),
                                Tab => "\x09".into(),
                                Up => "\x00H".into(),
                                Down => "\x00P".into(),
                                Left => "\x00K".into(),
                                Right => "\x00M".into(),
                                Delete => "\x00S".into(),
                                Insert => "\x00R".into(),
                                Home => "\x00G".into(),
                                End => "\x00O".into(),
                                PageUp => "\x00I".into(),
                                PageDown => "\x00Q".into(),
                                Char(c) => c.to_string().into(),
                                Ctrl(c) => match std::char::from_u32(c as u32 - 60) {
                                    Some(c) => c.to_string().into(),
                                    None => "".into(),
                                },
                                F(_) => "".into(),
                            };
                            break;
                        }
                        None => break,
                        _ => continue,
                    }
                }
                runtime.enter(&s);
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

fn save(listing: &Listing, filename: &str) -> Result<(), Error> {
    if listing.is_empty() {
        return Err(error!(InternalError; "NOTHING TO SAVE"));
    }
    let mut file = match fs::File::create(filename) {
        Ok(file) => file,
        Err(error) => return Err(error!(InternalError;  error.to_string().as_str())),
    };
    for line in listing.lines() {
        if let Err(error) = writeln!(file, "{}", line) {
            return Err(error!(InternalError; error.to_string().as_str()));
        }
    }
    Ok(())
}

fn parse_filename(filename: &str, index: usize) -> Result<String, Error> {
    let filename = filename.trim();
    if filename.len() < 3 || !filename.starts_with('"') || !filename.ends_with('"') {
        return Err(error!(BadFileName; &format!(
            "In line {} of the patch file.",
            index + 1
        )));
    }
    let filename = filename[1..filename.len() - 1].to_string();
    match fs::metadata(&filename) {
        Ok(_metadata) => {
            println!("Saving to {}", filename);
            Err(error!(FileAlreadyExists; &format!(
                "In line {} of the patch file.", index+1
            )))
        }
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                Ok(filename)
            } else {
                Err(error!(InternalError; &e.to_string()))
            }
        }
    }
}

fn load(filename: &str, allow_patch: bool, ignore_errors: bool) -> Result<Listing, Error> {
    if filename.starts_with("http://")
        || filename.starts_with("https://")
        || filename.starts_with("//")
    {
        let filename = if filename.starts_with("//") {
            let mut url =
                "https://raw.githubusercontent.com/AE9RB/basic-lang/master/patch/".to_string();
            url.push_str(&filename[2..]);
            url
        } else {
            filename.to_string()
        };
        let mut reader = match reqwest::blocking::get(&filename) {
            Ok(y) => {
                if y.status().is_success() {
                    BufReader::new(y)
                } else {
                    return Err(error!(FileNotFound; &format!("{}", y.status())));
                }
            }
            Err(e) => return Err(error!(InternalError; e.to_string().as_str())),
        };
        load2(&mut reader, allow_patch, ignore_errors)
    } else {
        let mut reader = match fs::File::open(filename) {
            Ok(file) => BufReader::new(file),
            Err(error) => {
                let msg = error.to_string();
                match error.kind() {
                    ErrorKind::NotFound => return Err(error!(FileNotFound; msg.as_str())),
                    _ => return Err(error!(InternalError; msg.as_str())),
                }
            }
        };
        load2(&mut reader, allow_patch, ignore_errors)
    }
}

fn load2(
    reader: &mut dyn std::io::BufRead,
    allow_patch: bool,
    ignore_errors: bool,
) -> Result<Listing, Error> {
    let mut first_listing = Listing::default();
    let mut listing = Listing::default();
    let mut patching = false;
    let mut filename = String::default();
    for (index, line) in reader.lines().enumerate() {
        match line {
            Err(error) => return Err(error!(InternalError; error.to_string().as_str())),
            Ok(line) => {
                if allow_patch && index == 0 && (line.starts_with('"') || line.starts_with('\'')) {
                    patching = true;
                    println!("Patch mode.\n");
                }
                if patching && line.starts_with('\'') {
                    println!("{}", line[1..].trim());
                    continue;
                }
                if patching && line.starts_with('"') {
                    let mut parts: Vec<&str> = line.split_ascii_whitespace().collect();
                    if parts.len() == 1 {
                        filename = parse_filename(parts.pop().unwrap(), index)?;
                    } else if parts.len() == 3 {
                        if !filename.is_empty() {
                            println!("Saving to {}", filename);
                            save(&listing, &filename)?;
                            println!();
                        }
                        if first_listing.is_empty() {
                            std::mem::swap(&mut listing, &mut first_listing)
                        }
                        let url = parts.pop().unwrap();
                        let crc = parts.pop().unwrap();
                        filename = parse_filename(parts.pop().unwrap(), index)?;
                        println!("Retrieving from {}", url);
                        listing = load(url, false, true)?;
                        let crc = match u32::from_str_radix(crc, 16) {
                            Ok(crc) => crc,
                            Err(_) => {
                                return Err(error!(SyntaxError; &format!(
                                    "Unable to parse crc info in line {} of the patch file.",
                                    index + 1
                                )));
                            }
                        };
                        let mut digest = crc::crc32::Digest::new(crc::crc32::IEEE);
                        for line in listing.lines() {
                            digest.write(line.to_string().as_bytes());
                        }
                        let digest = digest.sum32();
                        if digest != crc {
                            return Err(error!(SyntaxError; &format!(
                                "Expected CRC {:08X} got {:08X} in line {} of the patch file.",
                                crc, digest, index + 1
                            )));
                        }
                    } else {
                        return Err(error!(SyntaxError; &format!(
                            "Unable to parse info in line {} of the patch file.",
                            index + 1
                        )));
                    }
                    continue;
                }
                if let Err(error) = listing.load_str(&line) {
                    if !ignore_errors {
                        return Err(error.message(&format!("In line {} of the file.", index + 1)));
                    }
                }
            }
        }
    }
    if patching {
        println!("Saving to {}", filename);
        save(&listing, &filename)?;
        println!();
        if !first_listing.is_empty() {
            Ok(first_listing)
        } else {
            Ok(listing)
        }
    } else {
        Ok(listing)
    }
}
