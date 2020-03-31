use super::MAX_LINE_LEN;
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber, MaxValue};
use std::collections::{btree_map::Values, BTreeMap};
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct Listing {
    source: Arc<BTreeMap<LineNumber, Line>>,
    pub indirect_errors: Arc<Vec<Error>>,
    pub direct_errors: Arc<Vec<Error>>,
}

impl Listing {
    pub fn clear(&mut self) {
        self.source = Arc::default();
        self.indirect_errors = Arc::default();
        self.direct_errors = Arc::default();
    }

    pub fn insert(&mut self, line: Line) -> Option<Line> {
        Arc::get_mut(&mut self.source)
            .unwrap()
            .insert(line.number(), line)
    }

    pub fn remove(&mut self, ln: LineNumber) -> Option<Line> {
        Arc::get_mut(&mut self.source).unwrap().remove(&ln)
    }

    pub fn line(&self, num: usize) -> Option<(String, Vec<Range<usize>>)> {
        if num > LineNumber::max_value() as usize {
            return None;
        }
        let mut range = Some(num as u16)..Some(num as u16);
        self.list_line(&mut range)
    }

    pub fn lines(&self) -> Values<'_, LineNumber, Line> {
        self.source.values()
    }

    pub fn load_str(&mut self, line: &str) -> Result<(), Error> {
        if line.len() > MAX_LINE_LEN {
            return Err(error!(LineBufferOverflow));
        }
        let line = Line::new(line);
        let line_number = line.number();
        if line.is_empty() {
            return Ok(());
        }
        if line.is_direct() {
            Err(error!(IllegalDirect, line_number))
        } else {
            self.insert(line);
            Ok(())
        }
    }

    pub fn list_line(&self, range: &mut Range<LineNumber>) -> Option<(String, Vec<Range<usize>>)> {
        let mut source_range = self.source.range(range.start..=range.end);
        if let Some((line_number, line)) = source_range.next() {
            if *line_number < range.end {
                if let Some(num) = line_number {
                    range.start = Some(num + 1);
                }
            } else {
                range.start = Some(LineNumber::max_value() + 1);
                range.end = Some(LineNumber::max_value() + 1);
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
}
