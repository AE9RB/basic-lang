use super::{token::*, LineNumber, MaxValue};
use std::collections::VecDeque;

pub fn lex(source_line: &str) -> (LineNumber, Vec<Token>) {
    BasicLexer::lex(source_line)
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

struct BasicLexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pending: VecDeque<Token>,
    remark: bool,
}

impl<'a> Iterator for BasicLexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.pending.pop_front() {
            return Some(t);
        }
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
            let token = self.alphabetic();
            if matches!(token, Some(Token::Word(Word::Rem1))) {
                self.remark = true;
            }
            return token;
        }
        if *pk == '"' {
            return self.string();
        }
        if *pk == '&' {
            return self.radix();
        }
        let minutia = self.minutia();
        if matches!(minutia, Some(Token::Word(Word::Rem2))) {
            self.remark = true;
        }
        minutia
    }
}

impl<'a> BasicLexer<'a> {
    fn lex(mut source_line: &str) -> (LineNumber, Vec<Token>) {
        let mut line_number = None;
        let mut line_str_pos: usize = 0;
        let mut seen_digit = false;
        while let Some(s) = source_line.get(line_str_pos..) {
            if let Some(ch) = s.chars().next() {
                if seen_digit && is_basic_whitespace(ch) {
                    break;
                }
                if is_basic_digit(ch) {
                    seen_digit = true;
                } else if !is_basic_whitespace(ch) {
                    break;
                }
                line_str_pos += 1;
            } else {
                break;
            }
        }
        if let Ok(num) = source_line[0..line_str_pos].trim_start().parse::<u16>() {
            if num <= LineNumber::max_value() {
                line_number = Some(num);
                if let Some(' ') = source_line[line_str_pos..].chars().next() {
                    line_str_pos += 1;
                }
                source_line = &source_line[line_str_pos..];
            }
        }
        let mut tokens = BasicLexer {
            chars: source_line.chars().peekable(),
            pending: VecDeque::default(),
            remark: false,
        }
        .collect();
        BasicLexer::trim_end(&mut tokens);
        BasicLexer::collapse_triples(&mut tokens);
        BasicLexer::collapse_doubles(&mut tokens);
        BasicLexer::separate_words(&mut tokens);
        (line_number, tokens)
    }

    fn collapse_triples(tokens: &mut Vec<Token>) {
        let mut locs: Vec<(usize, Token)> = vec![];
        for (index, ttt) in tokens.windows(3).enumerate() {
            if let Token::Operator(Operator::Less) = &ttt[0] {
                if let Token::Whitespace(_) = &ttt[1] {
                    if let Token::Operator(Operator::Greater) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::NotEqual)));
                    }
                    if let Token::Operator(Operator::Equal) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::LessEqual)));
                    }
                }
            }
            if let Token::Operator(Operator::Equal) = &ttt[0] {
                if let Token::Whitespace(_) = &ttt[1] {
                    if let Token::Operator(Operator::Greater) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::GreaterEqual)));
                    }
                    if let Token::Operator(Operator::Less) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::LessEqual)));
                    }
                }
            }
            if let Token::Operator(Operator::Greater) = &ttt[0] {
                if let Token::Whitespace(_) = &ttt[1] {
                    if let Token::Operator(Operator::Less) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::NotEqual)));
                    }
                    if let Token::Operator(Operator::Equal) = &ttt[2] {
                        locs.push((index, Token::Operator(Operator::GreaterEqual)));
                    }
                }
            }
            if let Token::Ident(Ident::Plain(go)) = &ttt[0] {
                if go == "GO" {
                    if let Token::Whitespace(_) = ttt[1] {
                        if let Token::Word(Word::To) = ttt[2] {
                            locs.push((index, Token::Word(Word::Goto)));
                        }
                        if let Token::Ident(Ident::Plain(sub)) = &ttt[2] {
                            if sub == "SUB" {
                                locs.push((index, Token::Word(Word::Gosub)));
                            }
                        }
                    }
                }
            }
        }
        while let Some((index, token)) = locs.pop() {
            tokens.splice(index..index + 3, Some(token));
        }
    }

    fn collapse_doubles(tokens: &mut Vec<Token>) {
        let mut locs: Vec<(usize, Token)> = vec![];
        let mut tokens_iter = tokens.windows(2).enumerate();
        while let Some((index, tt)) = tokens_iter.next() {
            if let Token::Operator(Operator::Equal) = tt[0] {
                if let Token::Operator(Operator::Greater) = tt[1] {
                    locs.push((index, Token::Operator(Operator::GreaterEqual)));
                    tokens_iter.next();
                }
                if let Token::Operator(Operator::Less) = tt[1] {
                    locs.push((index, Token::Operator(Operator::LessEqual)));
                    tokens_iter.next();
                }
            }
            if let Token::Operator(Operator::Equal) = tt[1] {
                if let Token::Operator(Operator::Greater) = tt[0] {
                    locs.push((index, Token::Operator(Operator::GreaterEqual)));
                    tokens_iter.next();
                }
                if let Token::Operator(Operator::Less) = tt[0] {
                    locs.push((index, Token::Operator(Operator::LessEqual)));
                    tokens_iter.next();
                }
            }
            if let Token::Operator(Operator::Less) = tt[0] {
                if let Token::Operator(Operator::Greater) = tt[1] {
                    locs.push((index, Token::Operator(Operator::NotEqual)));
                    tokens_iter.next();
                }
            }
        }
        while let Some((index, token)) = locs.pop() {
            tokens.splice(index..index + 2, Some(token));
        }
    }

    fn separate_words(tokens: &mut Vec<Token>) {
        let mut locs: Vec<usize> = vec![];
        for (index, tt) in tokens.windows(2).enumerate() {
            if tt.iter().all(Token::is_word) {
                locs.push(index);
            }
        }
        while let Some(index) = locs.pop() {
            tokens.insert(index + 1, Token::Whitespace(1));
        }
    }

    fn trim_end(tokens: &mut Vec<Token>) {
        if let Some(Token::Whitespace(_)) = tokens.last() {
            tokens.pop();
        }
        if let Some(Token::Unknown(_)) = tokens.last() {
            if let Some(Token::Unknown(s)) = tokens.pop() {
                tokens.push(Token::Unknown(s.trim_end().into()));
            }
        }
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
        while let Some(mut ch) = self.chars.next() {
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
        if !exp && !decimal && s.parse::<i16>().is_ok() {
            return Some(Token::Literal(Literal::Integer(s)));
        }
        Some(Token::Literal(Literal::Single(s)))
    }

    fn string(&mut self) -> Option<Token> {
        let mut s = String::new();
        self.chars.next();
        while let Some(ch) = self.chars.next() {
            if ch == '"' {
                break;
            }
            s.push(ch);
        }
        Some(Token::Literal(Literal::String(s)))
    }

    fn alphabetic(&mut self) -> Option<Token> {
        let mut s = String::new();
        let mut digit = false;
        while let Some(ch) = self.chars.next() {
            let ch = ch.to_ascii_uppercase();
            s.push(ch);
            if is_basic_digit(ch) {
                digit = true;
            }
            if ch == '$' {
                self.pending.push_back(Token::Ident(Ident::String(s)));
                break;
            } else if ch == '!' {
                self.pending.push_back(Token::Ident(Ident::Single(s)));
                break;
            } else if ch == '#' {
                self.pending.push_back(Token::Ident(Ident::Double(s)));
                break;
            } else if ch == '%' {
                self.pending.push_back(Token::Ident(Ident::Integer(s)));
                break;
            }
            if let Some(pk) = self.chars.peek() {
                if is_basic_alphabetic(*pk) {
                    if digit {
                        self.pending.push_back(Token::Ident(Ident::Plain(s)));
                        break;
                    }
                    continue;
                }
                if is_basic_digit(*pk) || *pk == '$' || *pk == '!' || *pk == '#' || *pk == '%' {
                    s = Token::scan_alphabetic(&mut self.pending, &s);
                    if s.is_empty() {
                        break;
                    }
                    continue;
                }
            }
            s = Token::scan_alphabetic(&mut self.pending, &s);
            if !s.is_empty() {
                self.pending.push_back(Token::Ident(Ident::Plain(s)));
            }
            break;
        }
        self.pending.pop_front()
    }

    fn radix(&mut self) -> Option<Token> {
        self.chars.next();
        let is_hex = if matches!(self.chars.peek(), Some('H') | Some('h')) {
            self.chars.next();
            true
        } else {
            false
        };
        let mut s = String::new();
        while let Some(ch) = self.chars.next() {
            let ch = ch.to_ascii_uppercase();
            if ('0'..='7').contains(&ch)
                || (is_hex && (('8'..='9').contains(&ch) || ('A'..='F').contains(&ch)))
            {
                s.push(ch)
            } else {
                break;
            }
        }
        if is_hex {
            Some(Token::Literal(Literal::Hex(s)))
        } else {
            Some(Token::Literal(Literal::Octal(s)))
        }
    }

    fn minutia(&mut self) -> Option<Token> {
        let mut s = String::new();
        while let Some(ch) = self.chars.next() {
            s.push(ch);
            if let Some(token) = Token::match_minutia(&s) {
                return Some(token);
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
        Some(Token::Unknown(s))
    }
}
