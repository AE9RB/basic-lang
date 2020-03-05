use super::{Address, Val};

/// ## Virtual machine instruction set
///
/// The BASIC virtual machine has no registers.
/// Every operation is performed on the stack.
///
/// For example: `LET A=3*B` compiles to `[Literal(3), Push("B"), Mul, Pop("A")]`
///
/// See <https://en.wikipedia.org/wiki/Reverse_Polish_notation>

#[derive(Debug)]
pub enum Op {
    // *** Stack manipulation
    /// Push literal value on to the stack.
    Literal(Val),
    /// Push stack value of named variable. Infallible.
    Push(String),
    /// Pop stack value to named variable. This is the `LET` statement
    /// and may generate errors.
    Pop(String),

    // *** Branch control
    /// Expects Next(Address) on stack or else error: NEXT WITHOUT FOR.
    /// Then pops variable ident, step value, and to value.
    /// if wrong variable, repeat; to break out of a loop.
    /// Modify the variable using the step then check for end.
    /// If not done, push it all back on stack and jump to Address.
    Next(String),
    /// Pop stack and branch to Address if zero.
    If(Address),
    /// Pop stack and branch to Address if not zero.
    IfNot(Address),
    /// Unconditional branch to Address.
    Jump(Address),
    /// Expect Return(Address) on stack or else error: RETURN WITHOUT GOSUB.
    /// Branch to Address.
    Return,

    // *** Statements
    End,
    Print,

    // *** Expression operations
    Neg,
    Add,
    Sub,
    Mul,
    Div,

    // *** Built-in functions
    FnSin,
    FnCos,
    FnStrS,
}

/*
VM design notes. Move to docs some day.

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
