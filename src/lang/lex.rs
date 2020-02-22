use super::token::*;
use std::iter::Peekable;

pub fn lex(s: &str) -> (Option<u16>, Vec<Token>) {
    let mut t = s.len();
    if s.ends_with("\r\n") {
        t -= 2
    } else if s.ends_with("\n") {
        t -= 1
    }
    let l = &mut Lex {
        i: s.chars().take(t).peekable(),
        remark: false,
    };
    let mut t = l.collect::<Vec<Token>>();
    let line_number = take_line_number(&mut t);
    trim_end(&mut t);
    collapse_go(&mut t);
    if line_number.is_some() {
        separate_words(&mut t);
        upgrade_tokens(&mut t);
    }
    (line_number, t)
}

fn collapse_go(t: &mut Vec<Token>) {
    let mut ins: Vec<(usize, Token)> = vec![];
    for (i, ttt) in t.windows(3).enumerate() {
        if ttt[0] == Token::Ident(Ident::Plain("GO".to_string())) {
            if let Token::Whitespace(_) = ttt[1] {
                if ttt[2] == Token::Word(Word::To) {
                    ins.push((i, Token::Word(Word::Goto2)));
                }
                if ttt[2] == Token::Ident(Ident::Plain("SUB".to_string())) {
                    ins.push((i, Token::Word(Word::Gosub2)));
                }
            }
        }
    }
    while let Some((i, ttt)) = ins.pop() {
        t.drain(i..i + 3);
        t.insert(i, ttt);
    }
}

fn upgrade_tokens(t: &mut Vec<Token>) {
    for i in 0..t.len() {
        let r = match t[i] {
            Token::Word(Word::Print2) => Some(Token::Word(Word::Print1)),
            Token::Word(Word::Goto2) => Some(Token::Word(Word::Goto1)),
            Token::Word(Word::Gosub2) => Some(Token::Word(Word::Gosub1)),
            _ => None,
        };
        if r.is_some() {
            t[i] = r.unwrap();
        }
    }
}

fn separate_words(t: &mut Vec<Token>) {
    let mut ins: Vec<usize> = vec![];
    for (i, tt) in t.windows(2).enumerate() {
        let w1 = match tt[0] {
            Token::Word(_) => true,
            Token::Ident(_) => true,
            Token::Literal(_) => true,
            _ => false,
        };
        let w2 = match tt[1] {
            Token::Word(_) => true,
            Token::Ident(_) => true,
            Token::Literal(_) => true,
            _ => false,
        };
        if w1 && w2 {
            ins.push(i);
        }
    }
    while let Some(i) = ins.pop() {
        t.insert(i + 1, Token::Whitespace(1));
    }
}

fn trim_end(t: &mut Vec<Token>) {
    if let Some(Token::Whitespace(_)) = t.last() {
        t.pop();
    }
    if let Some(Token::Unknown(_)) = t.last() {
        if let Some(Token::Unknown(s)) = t.pop() {
            t.push(Token::Unknown(s.trim_end().to_string()));
        }
    }
}

fn take_line_number(t: &mut Vec<Token>) -> Option<u16> {
    let mut pos: Option<usize> = None;
    if let Some(Token::Literal(_)) = t.get(1) {
        if let Some(Token::Whitespace(_)) = t.get(0) {
            pos = Some(1);
        }
    } else if let Some(Token::Literal(_)) = t.get(0) {
        pos = Some(0);
    }
    if let Some(pos) = pos {
        let s = t.get(pos).unwrap();
        if let Token::Literal(lit) = s {
            let s = match lit {
                Literal::Integer(s) => s,
                Literal::Single(s) => s,
                Literal::Double(s) => s,
                Literal::String(s) => s,
            };
            if s.chars().all(|c| is_basic_digit(c)) {
                if let Ok(line) = s.parse::<u16>() {
                    if line <= 65529 {
                        t.drain(0..=pos);
                        let wsc: usize = match t.get(0) {
                            Some(Token::Whitespace(x)) => *x,
                            _ => 0,
                        };
                        if wsc == 1 {
                            t.remove(0);
                        }
                        if wsc > 1 {
                            t[0] = Token::Whitespace(wsc - 1);
                        }
                        return Some(line);
                    }
                }
            }
        }
    }
    None
}

fn is_basic_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_basic_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_basic_alphabetic(c: char) -> bool {
    c.is_ascii_alphabetic()
}

struct Lex<T: Iterator<Item = char>> {
    i: Peekable<T>,
    remark: bool,
}

impl<T: Iterator<Item = char>> Iterator for Lex<T> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.i.peek()?;
        if self.remark {
            return Some(Token::Unknown(self.i.by_ref().collect::<String>()));
        }
        if is_basic_whitespace(*p) {
            let tw = self.whitespace();
            return tw;
        }
        if is_basic_digit(*p) || *p == '.' {
            let tn = self.number();
            return tn;
        }
        if is_basic_alphabetic(*p) {
            let r = self.alphabetic();
            if r == Some(Token::Word(Word::Rem1)) {
                self.remark = true;
            }
            return r;
        }
        if *p == '"' {
            return self.string();
        }
        let r = self.minutia();
        if r == Some(Token::Word(Word::Rem2)) {
            self.remark = true;
        }
        return r;
    }
}

impl<T: Iterator<Item = char>> Lex<T> {
    fn whitespace(&mut self) -> Option<Token> {
        let mut spaces = 0;
        loop {
            self.i.next();
            spaces += 1;
            if let Some(p) = self.i.peek() {
                if is_basic_whitespace(*p) {
                    continue;
                }
            }
            return Some(Token::Whitespace(spaces));
        }
    }

    fn number(&mut self) -> Option<Token> {
        let mut s = String::new();
        let mut digits = 0;
        let mut decimal = false;
        let mut exp = false;
        loop {
            let mut c = self.i.next().unwrap();
            if c == 'e' {
                c = 'E'
            }
            if c == 'd' {
                c = 'D'
            }
            s.push(c);
            if !exp && is_basic_digit(c) {
                digits += 1;
            }
            if c == '.' {
                decimal = true
            }
            if c == 'D' {
                digits += 8;
            }
            if c == '!' {
                return Some(Token::Literal(Literal::Single(s)));
            }
            if c == '#' {
                return Some(Token::Literal(Literal::Double(s)));
            }
            if c == '%' {
                return Some(Token::Literal(Literal::Integer(s)));
            }
            if let Some(p) = self.i.peek() {
                if c == 'E' || c == 'D' {
                    exp = true;
                    if *p == '+' || *p == '-' {
                        continue;
                    }
                }
                if is_basic_digit(*p) {
                    continue;
                }
                if !decimal && *p == '.' {
                    continue;
                }
                if !exp && *p == 'E' || *p == 'e' || *p == 'D' || *p == 'd' {
                    continue;
                }
                if *p == '!' || *p == '#' || *p == '%' {
                    continue;
                }
            }
            if digits > 7 {
                return Some(Token::Literal(Literal::Double(s)));
            }
            if !exp && !decimal {
                if let Ok(_) = s.parse::<i16>() {
                    return Some(Token::Literal(Literal::Integer(s)));
                }
            }
            return Some(Token::Literal(Literal::Single(s)));
        }
    }

    fn string(&mut self) -> Option<Token> {
        let mut s = String::new();
        self.i.next();
        loop {
            if let Some(c) = self.i.next() {
                if c != '"' {
                    s.push(c);
                    continue;
                }
            }
            return Some(Token::Literal(Literal::String(s)));
        }
    }

    fn alphabetic(&mut self) -> Option<Token> {
        let mut s = String::new();
        let mut digit = false;
        loop {
            let c = self.i.next().unwrap().to_ascii_uppercase();
            s.push(c);
            if is_basic_digit(c) {
                digit = true;
            }
            if let Some(t) = Token::from_string(&s) {
                return Some(t);
            }
            if c == '$' {
                return Some(Token::Ident(Ident::String(s)));
            }
            if c == '!' {
                return Some(Token::Ident(Ident::Single(s)));
            }
            if c == '#' {
                return Some(Token::Ident(Ident::Double(s)));
            }
            if c == '%' {
                return Some(Token::Ident(Ident::Integer(s)));
            }
            if let Some(p) = self.i.peek() {
                if is_basic_alphabetic(*p) {
                    if digit {
                        break;
                    }
                    continue;
                }
                if is_basic_digit(*p) || *p == '$' || *p == '%' {
                    continue;
                }
            }
            break;
        }
        return Some(Token::Ident(Ident::Plain(s)));
    }

    fn minutia(&mut self) -> Option<Token> {
        let mut s = String::new();
        loop {
            if let Some(c) = self.i.next() {
                s.push(c);
                if let Some(t) = Token::from_string(&s) {
                    return Some(t);
                }
                if let Some(p) = self.i.peek() {
                    if is_basic_alphabetic(*p) {
                        break;
                    }
                    if is_basic_digit(*p) {
                        break;
                    }
                    if is_basic_whitespace(*p) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        return Some(Token::Unknown(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(s: &str) -> Token {
        let s = format!("?{}", s);
        let (_, l) = lex(&s);
        let mut i = l.iter();
        i.next();
        i.next().unwrap().clone()
    }

    #[test]
    fn test_go_to1() {
        let (ln, v) = lex("10 go to");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Goto1));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_go_to2() {
        assert_eq!(token("GO TO"), Token::Word(Word::Goto2));
    }

    #[test]
    fn test_go_sub2() {
        assert_eq!(token("GO SUB"), Token::Word(Word::Gosub2));
    }

    #[test]
    fn test_print1() {
        let (ln, v) = lex("10 ?");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_print2() {
        assert_eq!(token("?"), Token::Word(Word::Print2));
    }

    #[test]
    fn test_numbers() {
        assert_eq!(
            token("3.141593"),
            Token::Literal(Literal::Single("3.141593".to_string()))
        );
        assert_eq!(
            token("3.1415926"),
            Token::Literal(Literal::Double("3.1415926".to_string()))
        );
        assert_eq!(
            token("32767"),
            Token::Literal(Literal::Integer("32767".to_string()))
        );
        assert_eq!(
            token("32768"),
            Token::Literal(Literal::Single("32768".to_string()))
        );
        assert_eq!(
            token("24e9"),
            Token::Literal(Literal::Single("24E9".to_string()))
        );
    }

    #[test]
    fn test_annotated_numbers() {
        assert_eq!(
            token("12334567890!"),
            Token::Literal(Literal::Single("12334567890!".to_string()))
        );
        assert_eq!(
            token("0#"),
            Token::Literal(Literal::Double("0#".to_string()))
        );
        assert_eq!(
            token("24e9%"),
            Token::Literal(Literal::Integer("24E9%".to_string()))
        );
    }

    #[test]
    fn test_remark1() {
        let (ln, v) = lex("100 REM  A fortunate comment \n");
        assert_eq!(ln, Some(100));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Rem1));
        assert_eq!(
            x.next().unwrap(),
            &Token::Unknown("  A fortunate comment".to_string())
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_remark2() {
        let (ln, v) = lex("100  'The comment  \r\n");
        assert_eq!(ln, Some(100));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Rem2));
        assert_eq!(
            x.next().unwrap(),
            &Token::Unknown("The comment".to_string())
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_ident_with_word() {
        let (ln, v) = lex("BANDS");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Plain("BANDS".to_string()))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_for_loop() {
        let (ln, v) = lex("forI%=1to30");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::For));
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Integer("I%".to_string()))
        );
        assert_eq!(x.next().unwrap(), &Token::Operator(Operator::Equals));
        assert_eq!(
            x.next().unwrap(),
            &Token::Literal(Literal::Integer("1".to_string()))
        );
        assert_eq!(x.next().unwrap(), &Token::Word(Word::To));
        assert_eq!(
            x.next().unwrap(),
            &Token::Literal(Literal::Integer("30".to_string()))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_trim_start() {
        let (ln, v) = lex(" 10 PRINT 10");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
    }

    #[test]
    fn test_do_not_trim_start() {
        let (ln, v) = lex("  PRINT 10");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Whitespace(2));
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
    }

    #[test]
    fn test_empty() {
        let (ln, v) = lex("");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_line_number_only() {
        let (ln, v) = lex("10");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_unknown() {
        let (ln, v) = lex("10 for %w");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::For));
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
        assert_eq!(x.next().unwrap(), &Token::Unknown("%".to_string()));
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Plain("W".to_string()))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_direct_non_spacing() {
        let (ln, v) = lex("printJ");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Plain("J".to_string()))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_insert_spacing() {
        let (ln, v) = lex("10 printJ:printK");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Plain("J".to_string()))
        );
        assert_eq!(x.next().unwrap(), &Token::Colon);
        assert_eq!(x.next().unwrap(), &Token::Word(Word::Print1));
        assert_eq!(x.next().unwrap(), &Token::Whitespace(1));
        assert_eq!(
            x.next().unwrap(),
            &Token::Ident(Ident::Plain("K".to_string()))
        );
        assert_eq!(x.next(), None);
    }
}
