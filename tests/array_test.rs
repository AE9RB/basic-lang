mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_array_basics() {
    let mut r = Runtime::default();
    r.enter("10 DIM A$(100), X(10,10)");
    r.enter("20 A$(42)=\"THE ANSWER\"");
    r.enter("30 X(4,2)=2.7182818");
    r.enter("40 PRINT A$(42)+\"!\", X(4,2)");
    r.enter("run");
    assert_eq!(exec(&mut r), "THE ANSWER!    2.7182817 \n");
}
