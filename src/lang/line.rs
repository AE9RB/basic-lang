use super::*;

#[derive(Debug)]
pub struct Line {
    number: LineNumber,
    tokens: Vec<token::Token>,
}

impl Line {
    pub fn new(s: &str) -> Line {
        let (line_number, tokens) = lex(s);
        Line {
            number: line_number,
            tokens,
        }
    }

    pub fn number(&self) -> LineNumber {
        self.number
    }

    pub fn is_direct(&self) -> bool {
        self.number.is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn ast(&self) -> Result<Vec<ast::Statement>, Error> {
        parse(self.number, &self.tokens)
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = self.tokens.iter().map(|s| s.to_string()).collect();
        if let Some(number) = self.number {
            write!(f, "{} {}", number, s)
        } else {
            write!(f, "{}", s)
        }
    }
}

impl<'a> IntoIterator for &'a Line {
    type Item = &'a Line;
    type IntoIter = std::option::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        Some(self).into_iter()
    }
}
