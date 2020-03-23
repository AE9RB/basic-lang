mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_input_to_array() {
    let mut r = Runtime::default();
    r.enter("input a%,b(a%):print a%;: print b(2-a%);");
    assert_eq!(exec(&mut r), "? ");
    r.enter("1,2");
    assert_eq!(exec(&mut r), " 1  2 \n");
}
