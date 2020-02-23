use super::ast::*;
use super::error::*;
use super::token::*;

type Result<T> = std::result::Result<T, Error>;

pub fn parse(line_number: Option<u16>, tokens: &[Token]) -> Result<Vec<Statement>> {
    match Parse::tokens(tokens) {
        Err(e) => Err(e.in_line_number(line_number)),
        Ok(r) => Ok(r),
    }
}

struct Parse<'a> {
    token_stream: std::slice::Iter<'a, Token>,
    peeked: Option<&'a Token>,
    rem2: bool,
    col: std::ops::Range<usize>,
}

impl<'a> Parse<'a> {
    fn tokens(tokens: &'a [Token]) -> Result<Vec<Statement>> {
        let mut p = Parse {
            token_stream: tokens.iter(),
            peeked: None,
            rem2: false,
            col: 0..0,
        };

        let mut r: Vec<Statement> = vec![];
        loop {
            match p.statement() {
                Ok(s) => r.push(s),
                Err(e) => return Err(e.in_column(&p.col)),
            }
            println!("!p {:?}", p.peek());
            match p.peek() {
                None => return Ok(r),
                Some(t) => {
                    if *t == &Token::Colon {
                        p.next();
                    }
                }
            }
        }
    }

    fn next(&mut self) -> Option<&'a Token> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        loop {
            self.col.start = self.col.end;
            let t = self.token_stream.next()?;
            self.col.end += t.to_string().chars().count();
            if self.rem2 {
                continue;
            }
            match t {
                Token::Word(Word::Rem2) => {
                    self.rem2 = true;
                    continue;
                }
                Token::Whitespace(_) => continue,
                _ => return Some(t),
            }
        }
    }

    fn peek(&mut self) -> Option<&&Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }

    fn statement(&mut self) -> Result<Statement> {
        match self.next().unwrap() {
            Token::Word(w) => Statement::for_word(self, w),
            _ => error!(SyntaxError),
        }
    }

    fn expression(&mut self) -> Result<Expression> {
        match self.next() {
            Some(Token::Ident(i)) => Ok(Expression::Ident(i.clone())),
            _ => error!(SyntaxError),
        }
    }
}

impl Statement {
    fn for_word(parse: &mut Parse, word: &Word) -> Result<Statement> {
        use Word::*;
        match word {
            Let => Self::r#let(parse),
            _ => error!(SyntaxError),
        }
    }

    fn r#let(parse: &mut Parse) -> Result<Statement> {
        let ident = match parse.next() {
            Some(Token::Ident(i)) => i.clone(),
            _ => return error!(SyntaxError),
        };
        match parse.next() {
            Some(Token::Operator(Operator::Equals)) => {}
            _ => return error!(SyntaxError),
        };
        let expr = parse.expression()?;
        Ok(Statement::Let(ident, expr))
    }
}

#[cfg(test)]
mod tests {
    use super::super::lex::*;
    use super::*;

    fn parse_str(s: &str) -> Result<Vec<Statement>> {
        let (lin, tokens) = lex(s);
        parse(lin, &tokens)
    }

    #[test]
    fn test_let_foo_eq_bar() {
        let x = parse_str("letfoo=bar");
        assert_eq!(
            x.unwrap(),
            vec![Statement::Let(
                Ident::Plain("FOO".to_string()),
                Expression::Ident(Ident::Plain("BAR".to_string()))
            )]
        );
    }
}
