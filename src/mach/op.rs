/*

// let r = 10 + a% * 2
Literal(10)   // lhs+
Push("A%") // lhs*
Literal(2)    // rhs*
Mul           // rhs+
Add           // result
Pop("R")

// def fnx(a%, a$) = expr
:fnx
Pop("fnx.a$")
Pop("fnx.a%")
--eval expr
Return

// a$ = fnx(10, "foo")
Literal(10)
Literal("foo")
GoSub(:fnx)
Pop("a$")

// builtin function cos(3.14)
Literal(3.14)
FnCos

// print "hello" "world"
Literal("hello")
Literal("world")
Literal('\n')
Literal(3)
print -- pops len and reverse prints

// FOR A = _from TO _to STEP _step
--eval _from
Pop("A")
--eval _to
--eval _step (or Literal(1))
Literal("A")
Lieral(Next(:loop_inner))
:loop_inner
-- loop stuff
Next

// while _expr
:again
-- eval _expr
IfNot(:done)
-- loop stuff
GoTo(:again)
:done

//gosub
Literal(Return(:after))
GoTo(:thesub)
:after

// return
:thesub
-- stuff
Return

//if x then 100
-- eval x
If(:100)
-- stuff
:100

//if x then 100 else 200
-- eval x
If(:100)
GoTo(:200)
-- stuff
:100
-- stuff
:200

//if x then a=5 else b=6
-- eval x
IfNot(:else)
--exec a=5
GoTo(:finish)
:else
--exec b=6
:finish

*/

use super::val::Val;

pub type Address = usize;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Op {
    // Stack access
    Literal(Val), // push literal
    Push(String), // push value of named variable
    Pop(String),  // assign variable to popped value

    // Branch control
    If(Address),    // pop stack and branch if zero
    IfNot(Address), // pop stack and branch if not zero
    GoTo(Address),  // unconditional branch
    Return,         // expect Return(Address) on stack then branch
    Next,           /* expect Next(Address) on stack, pop var ident, pop step, pop to
                     * if wrong var name, repeat. enables breaking out of loop.
                     * do step. If not done, push it all back on stack and GoTo(Address).
                     */

    // Statements
    Print,

    // Expression operations
    Neg,
    Add,
    Sub,
    Mul,
    Div,

    // Built-in functions
    FnSin,
    FnCos,
    FnStrS,
}
