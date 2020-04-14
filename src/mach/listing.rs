use super::MAX_LINE_LEN;
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber, MaxValue};
use std::collections::{btree_map::Values, BTreeMap, HashMap};
use std::ops::{Range, RangeInclusive};
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

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    pub fn insert(&mut self, line: Line) -> Option<Line> {
        Arc::get_mut(&mut self.source)
            .unwrap()
            .insert(line.number(), line)
    }

    pub fn remove(&mut self, ln: LineNumber) -> Option<Line> {
        Arc::get_mut(&mut self.source).unwrap().remove(&ln)
    }

    pub fn remove_range(&mut self, range: RangeInclusive<LineNumber>) -> bool {
        let to_remove = self
            .source
            .range(range)
            .map(|(k, _)| *k)
            .collect::<Vec<LineNumber>>();
        if to_remove.is_empty() {
            return false;
        }
        let source = Arc::get_mut(&mut self.source).unwrap();
        for line_number in to_remove {
            source.remove(&line_number);
        }
        true
    }

    pub fn line(&self, num: usize) -> Option<(String, Vec<Range<usize>>)> {
        if num > LineNumber::max_value() as usize {
            return None;
        }
        let mut range = Some(num as u16)..=Some(num as u16);
        self.list_line(&mut range)
    }

    pub fn lines(&self) -> Values<'_, LineNumber, Line> {
        self.source.values()
    }

    /// Used for loading a new Listing from a file.
    pub fn load_str(&mut self, line: &str) -> Result<(), Error> {
        if line.len() > MAX_LINE_LEN {
            return Err(error!(LineBufferOverflow));
        }
        let line = Line::new(line);
        if line.is_empty() {
            if !line.is_direct() {
                Arc::get_mut(&mut self.source)
                    .unwrap()
                    .remove(&line.number());
            }
            Ok(())
        } else if line.is_direct() {
            Err(error!(DirectStatementInFile))
        } else {
            self.insert(line);
            Ok(())
        }
    }

    pub fn list_line(
        &self,
        range: &mut RangeInclusive<LineNumber>,
    ) -> Option<(String, Vec<Range<usize>>)> {
        let mut source_range = self.source.range(range.clone());
        if let Some((line_number, line)) = source_range.next() {
            if line_number < range.end() {
                if let Some(num) = line_number {
                    *range = Some(num + 1)..=*range.end();
                }
            } else {
                *range = Some(LineNumber::max_value() + 1)..=Some(LineNumber::max_value() + 1);
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

    pub fn renum(&mut self, new_start: u16, old_start: u16, step: u16) -> Result<(), Error> {
        let mut changes: HashMap<u16, u16> = HashMap::default();
        let mut old_end: u16 = LineNumber::max_value() + 1;
        let mut new_num = new_start;
        for (&ln, _) in self.source.iter() {
            let ln = match ln {
                Some(ln) => ln,
                None => return Err(error!(InternalError)),
            };
            if ln >= old_start {
                if old_end <= LineNumber::max_value() && old_end >= new_start {
                    return Err(error!(IllegalFunctionCall));
                }
                if new_num > LineNumber::max_value() {
                    return Err(error!(Overflow));
                }
                changes.insert(ln, new_num);
                new_num = match new_num.checked_add(step) {
                    Some(num) => num,
                    None => return Err(error!(Overflow)),
                };
            } else {
                old_end = ln;
            }
        }
        let mut new_source: BTreeMap<LineNumber, Line> = BTreeMap::default();
        for line in self.lines() {
            let line = line.renum(&changes);
            new_source.insert(line.number(), line);
        }
        self.source = Arc::from(new_source);
        Ok(())
    }
}
