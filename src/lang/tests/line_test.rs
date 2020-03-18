use super::super::token::*;
use super::*;

#[test]
fn test_direct() {
    let l = Line::new("run");
    assert_eq!(l.tokens, [Token::Word(Word::Run)]);
}

#[test]
fn test_indirect() {
    let l = Line::new("100 end");
    assert_eq!(l.number, Some(100));
    assert_eq!(l.tokens, [Token::Word(Word::End)]);
}
