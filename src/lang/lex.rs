use super::{token::*, LineNumber, MaxValue};

pub fn lex(s: &str) -> (LineNumber, Vec<Token>) {
    BasicLexer::lex(s)
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

trait Tokenizers<'a> {
    fn chars(&mut self) -> &mut std::iter::Peekable<std::str::Chars<'a>>;

    fn whitespace(&mut self) -> Option<Token> {
        let mut len = 0;
        loop {
            self.chars().next();
            len += 1;
            if let Some(pk) = self.chars().peek() {
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
            let mut ch = match self.chars().next() {
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
            if let Some(pk) = self.chars().peek() {
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
        self.chars().next();
        loop {
            if let Some(ch) = self.chars().next() {
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
            let ch = match self.chars().next() {
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
            if let Some(pk) = self.chars().peek() {
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
        Some(Token::Ident(Ident::Plain(s)))
    }

    fn minutia(&mut self) -> Option<Token> {
        let mut s = String::new();
        loop {
            if let Some(ch) = self.chars().next() {
                s.push(ch);
                if let Some(t) = Token::from_string(&s) {
                    return Some(t);
                }
                if let Some(pk) = self.chars().peek() {
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
        Some(Token::Unknown(s))
    }
}

struct BasicLexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    remark: bool,
}

impl<'a> Tokenizers<'a> for BasicLexer<'a> {
    fn chars(&mut self) -> &mut std::iter::Peekable<std::str::Chars<'a>> {
        &mut self.chars
    }
}

impl<'a> Iterator for BasicLexer<'a> {
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
            if let Some(Token::Word(Word::Rem1)) = r {
                self.remark = true;
            }
            return r;
        }
        if *pk == '"' {
            return self.string();
        }
        let minutia = self.minutia();
        if let Some(Token::Word(Word::Rem2)) = minutia {
            self.remark = true;
        }
        minutia
    }
}

impl<'a> BasicLexer<'a> {
    fn lex(s: &str) -> (LineNumber, Vec<Token>) {
        let mut line_number = None;
        let mut s = s;
        let mut ln: usize = 0;
        let mut seen_digit = false;
        while let Some(n) = s.get(ln..) {
            if let Some(ch) = n.chars().next() {
                if seen_digit && is_basic_whitespace(ch) {
                    break;
                }
                if is_basic_digit(ch) {
                    seen_digit = true;
                } else if !is_basic_whitespace(ch) {
                    break;
                }
                ln += 1;
            } else {
                break;
            }
        }
        if let Ok(n) = s[0..ln].trim_start().parse::<u16>() {
            if n <= LineNumber::max_value() {
                line_number = Some(n);
                if let Some(' ') = s[ln..].chars().next() {
                    ln += 1;
                }
                s = &s[ln..];
            }
        }
        let mut tokens = BasicLexer {
            chars: s.chars().peekable(),
            remark: false,
        }
        .collect();
        BasicLexer::trim_end(&mut tokens);
        BasicLexer::collapse_go(&mut tokens);
        BasicLexer::collapse_lt_gt_equal(&mut tokens);
        if line_number.is_some() {
            BasicLexer::separate_words(&mut tokens);
            BasicLexer::upgrade_tokens(&mut tokens);
        }
        (line_number, tokens)
    }

    fn collapse_lt_gt_equal(tokens: &mut Vec<Token>) {
        let mut locs: Vec<(usize, Token)> = vec![];
        let mut tokens_iter = tokens.windows(2).enumerate();
        while let Some((index, tt)) = tokens_iter.next() {
            if let Token::Operator(Operator::Equal) = tt[0] {
                if let Token::Operator(Operator::Greater) = tt[1] {
                    locs.push((index, Token::Operator(Operator::EqualGreater)));
                    tokens_iter.next();
                }
                if let Token::Operator(Operator::Less) = tt[1] {
                    locs.push((index, Token::Operator(Operator::EqualLess)));
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

    fn collapse_go(tokens: &mut Vec<Token>) {
        let mut locs: Vec<(usize, Token)> = vec![];
        for (index, ttt) in tokens.windows(3).enumerate() {
            if let Token::Ident(Ident::Plain(go)) = &ttt[0] {
                if go == "GO" {
                    if let Token::Whitespace(_) = ttt[1] {
                        if let Token::Word(Word::To) = ttt[2] {
                            locs.push((index, Token::Word(Word::Goto2)));
                        }
                        if let Token::Ident(Ident::Plain(sub)) = &ttt[2] {
                            if sub == "SUB" {
                                locs.push((index, Token::Word(Word::Gosub2)));
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
}
