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
        let mut parse = Parse {
            token_stream: tokens.iter(),
            peeked: None,
            rem2: false,
            col: 0..0,
        };
        let mut r: Vec<Statement> = vec![];
        loop {
            match parse.peek() {
                None => return Ok(r),
                Some(t) => {
                    if *t == &Token::Colon {
                        parse.next();
                        continue;
                    }
                }
            }
            match parse.statement() {
                Ok(s) => r.push(s),
                Err(e) => return Err(e.in_column(&parse.col)),
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

    fn peek(&mut self) -> Option<&&'a Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }

    fn statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Some(Token::Ident(_)) => Statement::for_word(self, &Word::Let),
            Some(Token::Word(word)) => {
                self.next();
                Statement::for_word(self, word)
            }
            _ => error!(SyntaxError),
        }
    }

    fn expression(&mut self) -> Result<Expression> {
        match self.next() {
            Some(Token::Ident(i)) => match self.peek() {
                Some(&&Token::ParenOpen) => {
                    Ok(Expression::Function(i.clone(), self.expression_list()?))
                }
                _ => Ok(Expression::Ident(i.clone())),
            },
            Some(Token::Literal(l)) => Expression::for_literal(l),
            _ => error!(SyntaxError),
        }
    }

    fn expression_list(&mut self) -> Result<Vec<Expression>> {
        match self.next() {
            Some(Token::ParenOpen) => {}
            _ => return error!(SyntaxError),
        }
        let mut v: Vec<Expression> = vec![];
        loop {
            v.push(self.expression()?);
            match self.next() {
                Some(Token::ParenClose) => return Ok(v),
                Some(Token::Comma) => continue,
                _ => return error!(SyntaxError),
            }
        }
    }
}

impl Expression {
    fn for_literal(lit: &Literal) -> Result<Expression> {
        match lit {
            Literal::Single(s) => {
                let v = Self::clean(s).parse::<f32>();
                match v {
                    Ok(v) => Ok(Expression::Single(v)),
                    Err(why) => panic!(why),
                }
            }
            Literal::Double(s) => {
                let v = Self::clean(s).parse::<f64>();
                match v {
                    Ok(v) => Ok(Expression::Double(v)),
                    Err(why) => panic!(why),
                }
            }
            Literal::Integer(s) => {
                let v = Self::clean(s).parse::<i16>();
                match v {
                    Ok(v) => Ok(Expression::Integer(v)),
                    Err(why) => panic!(why),
                }
            }
            Literal::String(s) => Ok(Expression::String(s.to_string())),
        }
    }

    fn clean(s: &str) -> String {
        let mut s = String::from(s).replace("D", "E");
        match s.chars().last() {
            Some('!') | Some('#') | Some('%') => {
                s.pop();
            }
            _ => {}
        };
        s
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

    fn parse_str(s: &str) -> Statement {
        let (lin, tokens) = lex(s);
        match parse(lin, &tokens) {
            Ok(mut v) => {
                if v.len() != 1 {
                    panic!();
                }
                v.pop().unwrap()
            }
            Err(e) => panic!("{} : {:?}", e, e.column()),
        }
    }

    #[test]
    fn test_let_foo_eq_bar() {
        let answer = Statement::Let(
            Ident::Plain("FOO".to_string()),
            Expression::Ident(Ident::Plain("BAR".to_string())),
        );
        assert_eq!(parse_str("letfoo=bar:"), answer);
        assert_eq!(parse_str("foo=bar:"), answer);
    }

    #[test]
    fn test_literals() {
        let answer = Statement::Let(Ident::Plain("A".to_string()), Expression::Integer(12));
        assert_eq!(parse_str("A=12"), answer);
        let answer = Statement::Let(Ident::Plain("A".to_string()), Expression::Single(12.0));
        assert_eq!(parse_str("A=12!"), answer);
        let answer = Statement::Let(Ident::Plain("A".to_string()), Expression::Double(12e4));
        assert_eq!(parse_str("A=12d4"), answer);
        let answer = Statement::Let(
            Ident::Plain("A".to_string()),
            Expression::String("food".to_string()),
        );
        assert_eq!(parse_str("A=\"food\""), answer);
        let answer = Statement::Let(Ident::Plain("A".to_string()), Expression::Double(0.0));
        assert_eq!(
            parse_str("A=798347598234765983475983248592d-234721398742391847982344"),
            answer
        );
    }

    #[test]
    fn test_functions() {
        let answer = Statement::Let(
            Ident::Plain("A".to_string()),
            Expression::Function(
                Ident::Plain("COS".to_string()),
                vec![Expression::Single(3.14)],
            ),
        );
        assert_eq!(parse_str("A=cos(3.14)"), answer);
    }
}
