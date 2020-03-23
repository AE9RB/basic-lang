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
    pub fn lines(&self) -> Values<'_, LineNumber, Line> {
        self.source.values()
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

    pub fn list_line(&self, range: &mut Range<LineNumber>) -> Option<(String, Vec<Range<usize>>)> {
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
}
