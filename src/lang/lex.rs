use super::{token::*, LineNumber, MaxValue};
use std::convert::TryFrom;

pub fn lex(s: &str) -> (LineNumber, Vec<Token>) {
    let mut tokens = Lexer::lex(s);
    let line_number = take_line_number(&mut tokens);
    trim_end(&mut tokens);
    collapse_go(&mut tokens);
    collapse_lt_gt_equal(&mut tokens);
    if line_number.is_some() {
        separate_words(&mut tokens);
        upgrade_tokens(&mut tokens);
    }
    (line_number, tokens)
}

fn collapse_lt_gt_equal(tokens: &mut Vec<Token>) {
    let mut locs: Vec<(usize, Token)> = vec![];
    let mut tokens_iter = tokens.windows(2).enumerate();
    while let Some((index,tt)) = tokens_iter.next() {
        if tt[0] == Token::Operator(Operator::Equal) {
            if tt[1] == Token::Operator(Operator::Greater) {
                locs.push((index, Token::Operator(Operator::EqualGreater)));
                tokens_iter.next();
            }
            if tt[1] == Token::Operator(Operator::Less) {
                locs.push((index, Token::Operator(Operator::EqualLess)));
                tokens_iter.next();
            }
        }
        if tt[1] == Token::Operator(Operator::Equal) {
            if tt[0] == Token::Operator(Operator::Greater) {
                locs.push((index, Token::Operator(Operator::GreaterEqual)));
                tokens_iter.next();
            }
            if tt[0] == Token::Operator(Operator::Less) {
                locs.push((index, Token::Operator(Operator::LessEqual)));
                tokens_iter.next();
            }
        }
        if tt[0] == Token::Operator(Operator::Less) {
            if tt[1] == Token::Operator(Operator::Greater) {
                locs.push((index, Token::Operator(Operator::NotEqual)));
                tokens_iter.next();
            }
        }
    };
    while let Some((index, token)) = locs.pop() {
        tokens.splice(index..index + 2, Some(token));
    }
}

fn collapse_go(tokens: &mut Vec<Token>) {
    let mut locs: Vec<(usize, Token)> = vec![];
    for (index, ttt) in tokens.windows(3).enumerate() {
        if ttt[0] == Token::Ident(Ident::Plain("GO".to_string())) {
            if let Token::Whitespace(_) = ttt[1] {
                if ttt[2] == Token::Word(Word::To) {
                    locs.push((index, Token::Word(Word::Goto2)));
                }
                if ttt[2] == Token::Ident(Ident::Plain("SUB".to_string())) {
                    locs.push((index, Token::Word(Word::Gosub2)));
                }
            }
        }
    }
    while let Some((index, token)) = locs.pop() {
        tokens.splice(index..index + 3, Some(token));
    }
}

fn upgrade_tokens(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        match token {
            Token::Word(Word::Print2) => *token = Token::Word(Word::Print1),
            Token::Word(Word::Goto2) => *token = Token::Word(Word::Goto1),
            Token::Word(Word::Gosub2) => *token = Token::Word(Word::Gosub1),
            _ => {}
        };
    }
}

fn separate_words(tokens: &mut Vec<Token>) {
    let mut ins: Vec<usize> = vec![];
    for (index, tt) in tokens.windows(2).enumerate() {
        if tt.iter().all(|y| y.is_word()) {
            ins.push(index);
        }
    }
    while let Some(index) = ins.pop() {
        tokens.insert(index + 1, Token::Whitespace(1));
    }
}

fn trim_end(tokens: &mut Vec<Token>) {
    if let Some(Token::Whitespace(_)) = tokens.last() {
        tokens.pop();
    }
    if let Some(Token::Unknown(_)) = tokens.last() {
        if let Some(Token::Unknown(s)) = tokens.pop() {
            tokens.push(Token::Unknown(s.trim_end().to_string()));
        }
    }
}

fn take_line_number(tokens: &mut Vec<Token>) -> LineNumber {
    let mut pos: Option<usize> = None;
    if let Some(Token::Literal(_)) = tokens.get(1) {
        if let Some(Token::Whitespace(_)) = tokens.get(0) {
            pos = Some(1);
        }
    } else if let Some(Token::Literal(_)) = tokens.get(0) {
        pos = Some(0);
    }
    if let Some(pos) = pos {
        if let Some(token) = tokens.get(pos) {
            if let Ok(line) = LineNumber::try_from(token) {
                if let Some(val) = line {
                    if val <= LineNumber::max_value() {
                        tokens.drain(0..=pos);
                        let whitespace_len: usize = match tokens.get(0) {
                            Some(Token::Whitespace(len)) => *len,
                            _ => 0,
                        };
                        if whitespace_len == 1 {
                            tokens.remove(0);
                        }
                        if whitespace_len > 1 {
                            if let Some(token) = tokens.get_mut(0) {
                                *token = Token::Whitespace(whitespace_len - 1);
                            }
                        }
                        return line;
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

struct Lexer<'a> {
    chars: std::iter::Peekable<std::iter::Take<std::str::Chars<'a>>>,
    remark: bool,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let pk = self.chars.peek()?;
        if self.remark {
            return Some(Token::Unknown(self.chars.by_ref().collect::<String>()));
        }
        if is_basic_whitespace(*pk) {
            return self.whitespace();
        }
        if is_basic_digit(*pk) || *pk == '.' {
            return self.number();
        }
        if is_basic_alphabetic(*pk) {
            let r = self.alphabetic();
            if r == Some(Token::Word(Word::Rem1)) {
                self.remark = true;
            }
            return r;
        }
        if *pk == '"' {
            return self.string();
        }
        let r = self.minutia();
        if r == Some(Token::Word(Word::Rem2)) {
            self.remark = true;
        }
        return r;
    }
}

impl<'a> Lexer<'a> {
    fn lex(s: &str) -> Vec<Token> {
        let mut take = s.len();
        if s.ends_with("\r\n") {
            take -= 2
        } else if s.ends_with("\n") {
            take -= 1
        }
        Lexer {
            chars: s.chars().take(take).peekable(),
            remark: false,
        }
        .collect()
    }

    fn whitespace(&mut self) -> Option<Token> {
        let mut len = 0;
        loop {
            self.chars.next();
            len += 1;
            if let Some(pk) = self.chars.peek() {
                if is_basic_whitespace(*pk) {
                    continue;
                }
            }
            return Some(Token::Whitespace(len));
        }
    }

    fn number(&mut self) -> Option<Token> {
        let mut s = String::new();
        let mut digits = 0;
        let mut decimal = false;
        let mut exp = false;
        loop {
            let mut ch = match self.chars.next() {
                Some(c) => c,
                None => {
                    debug_assert!(false, "Failed to tokenize number.");
                    return None;
                }
            };
            if ch == 'e' {
                ch = 'E'
            }
            if ch == 'd' {
                ch = 'D'
            }
            s.push(ch);
            if !exp && is_basic_digit(ch) {
                digits += 1;
            }
            if ch == '.' {
                decimal = true
            }
            if ch == 'D' {
                digits += 8;
            }
            if ch == '!' {
                return Some(Token::Literal(Literal::Single(s)));
            }
            if ch == '#' {
                return Some(Token::Literal(Literal::Double(s)));
            }
            if ch == '%' {
                return Some(Token::Literal(Literal::Integer(s)));
            }
            if let Some(pk) = self.chars.peek() {
                if ch == 'E' || ch == 'D' {
                    exp = true;
                    if *pk == '+' || *pk == '-' {
                        continue;
                    }
                }
                if is_basic_digit(*pk) {
                    continue;
                }
                if !decimal && *pk == '.' {
                    continue;
                }
                if !exp && *pk == 'E' || *pk == 'e' || *pk == 'D' || *pk == 'd' {
                    continue;
                }
                if *pk == '!' || *pk == '#' || *pk == '%' {
                    continue;
                }
            }
            break;
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

    fn string(&mut self) -> Option<Token> {
        let mut s = String::new();
        self.chars.next();
        loop {
            if let Some(ch) = self.chars.next() {
                if ch != '"' {
                    s.push(ch);
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
            let ch = match self.chars.next() {
                Some(ch) => ch.to_ascii_uppercase(),
                None => {
                    debug_assert!(false, "Failed to tokenize alphabetic.");
                    return None;
                }
            };
            s.push(ch);
            if is_basic_digit(ch) {
                digit = true;
            }
            if let Some(token) = Token::from_string(&s) {
                return Some(token);
            }
            if ch == '$' {
                return Some(Token::Ident(Ident::String(s)));
            }
            if ch == '!' {
                return Some(Token::Ident(Ident::Single(s)));
            }
            if ch == '#' {
                return Some(Token::Ident(Ident::Double(s)));
            }
            if ch == '%' {
                return Some(Token::Ident(Ident::Integer(s)));
            }
            if let Some(pk) = self.chars.peek() {
                if is_basic_alphabetic(*pk) {
                    if digit {
                        break;
                    }
                    continue;
                }
                if is_basic_digit(*pk) || *pk == '$' || *pk == '!' || *pk == '#' || *pk == '%' {
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
            if let Some(ch) = self.chars.next() {
                s.push(ch);
                if let Some(t) = Token::from_string(&s) {
                    return Some(t);
                }
                if let Some(pk) = self.chars.peek() {
                    if is_basic_alphabetic(*pk) {
                        break;
                    }
                    if is_basic_digit(*pk) {
                        break;
                    }
                    if is_basic_whitespace(*pk) {
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

    fn token(s: &str) -> Option<Token> {
        let s = format!("?{}", s);
        let (_, mut tokens) = lex(&s);
        let mut t = tokens.drain(1..2);
        t.next()
    }

    #[test]
    fn test_eq_gt() {
        let (ln, v) = lex("10 1=<>=<>2");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(
            x.next(),
            Some(&Token::Literal(Literal::Integer("1".to_string())))
        );
        assert_eq!(x.next(), Some(&Token::Operator(Operator::EqualLess)));
        assert_eq!(x.next(), Some(&Token::Operator(Operator::GreaterEqual)));
        assert_eq!(x.next(), Some(&Token::Operator(Operator::NotEqual)));
        assert_eq!(
            x.next(),
            Some(&Token::Literal(Literal::Integer("2".to_string())))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_go_to_1() {
        let (ln, v) = lex("10 go to");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Goto1)));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_go_to_2() {
        assert_eq!(token("GO TO"), Some(Token::Word(Word::Goto2)));
    }

    #[test]
    fn test_go_sub_2() {
        assert_eq!(token("GO SUB"), Some(Token::Word(Word::Gosub2)));
    }

    #[test]
    fn test_print_1() {
        let (ln, v) = lex("10 ?");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_print_2() {
        assert_eq!(token("?"), Some(Token::Word(Word::Print2)));
    }

    #[test]
    fn test_numbers() {
        assert_eq!(
            token("3.141593"),
            Some(Token::Literal(Literal::Single("3.141593".to_string())))
        );
        assert_eq!(
            token("3.1415926"),
            Some(Token::Literal(Literal::Double("3.1415926".to_string())))
        );
        assert_eq!(
            token("32767"),
            Some(Token::Literal(Literal::Integer("32767".to_string())))
        );
        assert_eq!(
            token("32768"),
            Some(Token::Literal(Literal::Single("32768".to_string())))
        );
        assert_eq!(
            token("24e9"),
            Some(Token::Literal(Literal::Single("24E9".to_string())))
        );
    }

    #[test]
    fn test_annotated_numbers() {
        assert_eq!(
            token("12334567890!"),
            Some(Token::Literal(Literal::Single("12334567890!".to_string())))
        );
        assert_eq!(
            token("0#"),
            Some(Token::Literal(Literal::Double("0#".to_string())))
        );
        assert_eq!(
            token("24e9%"),
            Some(Token::Literal(Literal::Integer("24E9%".to_string())))
        );
    }

    #[test]
    fn test_remark1() {
        let (ln, v) = lex("100 REM  A fortunate comment \n");
        assert_eq!(ln, Some(100));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Rem1)));
        assert_eq!(
            x.next(),
            Some(&Token::Unknown("  A fortunate comment".to_string()))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_remark2() {
        let (ln, v) = lex("100  'The comment  \r\n");
        assert_eq!(ln, Some(100));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
        assert_eq!(x.next(), Some(&Token::Word(Word::Rem2)));
        assert_eq!(x.next(), Some(&Token::Unknown("The comment".to_string())));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_ident_with_word() {
        let (ln, v) = lex("BANDS");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(
            x.next(),
            Some(&Token::Ident(Ident::Plain("BANDS".to_string())))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_for_loop() {
        let (ln, v) = lex("forI%=1to30");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::For)));
        assert_eq!(
            x.next(),
            Some(&Token::Ident(Ident::Integer("I%".to_string())))
        );
        assert_eq!(x.next(), Some(&Token::Operator(Operator::Equal)));
        assert_eq!(
            x.next(),
            Some(&Token::Literal(Literal::Integer("1".to_string())))
        );
        assert_eq!(x.next(), Some(&Token::Word(Word::To)));
        assert_eq!(
            x.next(),
            Some(&Token::Literal(Literal::Integer("30".to_string())))
        );
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_trim_start() {
        let (ln, v) = lex(" 10 PRINT 10");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    }

    #[test]
    fn test_do_not_trim_start() {
        let (ln, v) = lex("  PRINT 10");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Whitespace(2)));
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
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
    fn test_string_at_start() {
        let (ln, v) = lex("\"HELLO\"");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(
            x.next(),
            Some(&Token::Literal(Literal::String("HELLO".to_string())))
        );
    }

    #[test]
    fn test_unknown() {
        let (ln, v) = lex("10 for %w");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::For)));
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
        assert_eq!(x.next(), Some(&Token::Unknown("%".to_string())));
        assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("W".to_string()))));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_direct_non_spacing() {
        let (ln, v) = lex("printJ");
        assert_eq!(ln, None);
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("J".to_string()))));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn test_insert_spacing() {
        let (ln, v) = lex("10 printJ:printK");
        assert_eq!(ln, Some(10));
        let mut x = v.iter();
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
        assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("J".to_string()))));
        assert_eq!(x.next(), Some(&Token::Colon));
        assert_eq!(x.next(), Some(&Token::Word(Word::Print1)));
        assert_eq!(x.next(), Some(&Token::Whitespace(1)));
        assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("K".to_string()))));
        assert_eq!(x.next(), None);
    }
}
