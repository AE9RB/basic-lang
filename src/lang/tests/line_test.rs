use super::super::token::*;
use super::*;

#[test]
fn test_lf() {
    let l = Line::new("run\n");
    assert_eq!(l.tokens, [Token::Word(Word::Run)]);
}

#[test]
fn test_crlf() {
    let l = Line::new("list\r\n");
    assert_eq!(l.tokens, [Token::Word(Word::List)]);
}
