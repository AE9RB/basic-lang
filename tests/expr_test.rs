mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_precedence() {
    let mut r = Runtime::default();
    r.enter("?1+2*3");
    assert_eq!(run(&mut r), " 7 \n");
    r.enter("?(1+2)*3");
    assert_eq!(run(&mut r), " 9 \n");
}

#[test]
fn test_variables() {
    let mut r = Runtime::default();
    r.enter("a=1+2*3:?a*2");
    assert_eq!(run(&mut r), " 14 \n");
    r.enter("a%=300*300");
    assert_eq!(run(&mut r), "?OVERFLOW\n");
}
