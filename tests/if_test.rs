mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_if_then() {
    let mut r = Runtime::default();
    r.enter("if 1 then ? \"one\"");
    assert_eq!(exec(&mut r), "one\n");
}

#[test]
fn test_if_then_else() {
    let mut r = Runtime::default();

    r.enter("if 0 then ? \"one\" else ? \"two\";:?2");
    assert_eq!(exec(&mut r), "two 2 \n");
    r.enter("if 1 then ? \"one\" else ? \"two\":?2");
    assert_eq!(exec(&mut r), "one\n");
}
