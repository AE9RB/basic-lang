use super::{Address, Val};
use std::rc::Rc;

/// ## Virtual machine instruction set
///
/// The BASIC virtual machine has no registers.
/// Every operation is performed on the stack.
///
/// For example: `LET A=3*B` compiles to `[Literal(3), Push(B), Mul, Pop(A)]`
///
/// See <https://en.wikipedia.org/wiki/Reverse_Polish_notation>

#[derive(Clone)]
pub enum Opcode {
    // *** Stack manipulation
    /// Push literal value on to the stack.
    Literal(Val),
    /// Push stack value of named variable. Infallible.
    Push(Rc<str>),
    /// Pop stack value to named variable. This is the `LET` statement
    /// and may generate errors.
    Pop(Rc<str>),
    PushArr(Rc<str>),
    PopArr(Rc<str>),
    DimArr(Rc<str>),
    EraseArr(Rc<str>),

    // *** Branch control
    /// Pop stack and branch to Address if not zero.
    IfNot(Address),
    /// Unconditional branch to Address.
    Jump(Address),
    /// Process the FOR loop on the stack.
    Next(Rc<str>),
    /// ON x GOTO/GOSUB lines
    On,
    /// Expect Return(Address) on stack or else error: RETURN WITHOUT GOSUB.
    /// A single assignable value before the Return(Address) will be restored to the stack.
    /// Branch to Address.
    Return,

    // *** Statements
    Clear,
    Cls,
    Cont,
    Def(Rc<str>),
    Defdbl,
    Defint,
    Defsng,
    Defstr,
    Delete,
    End,
    Fn(Rc<str>),
    Input(Rc<str>),
    List,
    Load,
    New,
    Print,
    Read,
    Restore(Address),
    Save,
    Stop,

    // *** Expression operations
    Neg,
    Pow,
    Mul,
    Div,
    DivInt,
    Mod,
    Add,
    Sub,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Not,
    And,
    Or,
    Xor,
    Imp,
    Eqv,

    // *** Built-in functions
    Abs,
    Asc,
    Atn,
    Cdbl,
    Chr,
    Cint,
    Cos,
    Csng,
    Date,
    Exp,
    Fix,
    Hex,
    Inkey,
    Instr,
    Int,
    Left,
    Len,
    Log,
    Mid,
    Oct,
    Pos,
    Right,
    Rnd,
    Sgn,
    Sin,
    Spc,
    Sqr,
    Str,
    String,
    Tab,
    Tan,
    Time,
    Val,
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Opcode::*;
        match self {
            Literal(v) => write!(f, "PUSH({})", format!("{:?}", v).to_ascii_uppercase()),
            Push(s) => write!(f, "PUSH({})", s),
            Pop(s) => write!(f, "POP({})", s),
            PushArr(s) => write!(f, "PUSHARR({})", s),
            PopArr(s) => write!(f, "POPARR({})", s),
            DimArr(s) => write!(f, "DIMARR({})", s),
            EraseArr(s) => write!(f, "ERASEARR({})", s),

            IfNot(a) => write!(f, "IFNOT({})", a),
            Jump(a) => write!(f, "JUMP({})", a),
            Next(a) => write!(f, "NEXT({})", a),
            On => write!(f, "ON"),
            Return => write!(f, "RETURN"),

            Clear => write!(f, "CLEAR"),
            Cls => write!(f, "CLS"),
            Cont => write!(f, "CONT"),
            Def(s) => write!(f, "DEF({})", s),
            Defdbl => write!(f, "DEFDBL"),
            Defint => write!(f, "DEFINT"),
            Defsng => write!(f, "DEFSNG"),
            Defstr => write!(f, "DEFSTR"),
            Delete => write!(f, "DELETE"),
            End => write!(f, "END"),
            Fn(s) => write!(f, "FN({})", s),
            Input(s) => write!(f, "INPUT({})", s),
            List => write!(f, "LIST"),
            New => write!(f, "NEW"),
            Load => write!(f, "LOAD"),
            Print => write!(f, "PRINT"),
            Read => write!(f, "READ"),
            Restore(s) => write!(f, "RESTORE({})", s),
            Save => write!(f, "SAVE"),
            Stop => write!(f, "STOP"),

            Neg => write!(f, "NEG"),
            Pow => write!(f, "POW"),
            Mul => write!(f, "MUL"),
            Div => write!(f, "DIV"),
            DivInt => write!(f, "DIVINT"),
            Mod => write!(f, "MOD"),
            Add => write!(f, "ADD"),
            Sub => write!(f, "SUB"),
            Eq => write!(f, "EQ"),
            NotEq => write!(f, "NOTEQ"),
            Lt => write!(f, "LT"),
            LtEq => write!(f, "LTEQ"),
            Gt => write!(f, "GT"),
            GtEq => write!(f, "GTEQ"),
            Not => write!(f, "NOT"),
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Xor => write!(f, "XOR"),
            Imp => write!(f, "IMP"),
            Eqv => write!(f, "EQV"),

            Abs => write!(f, "ABS"),
            Asc => write!(f, "ASC"),
            Atn => write!(f, "ATN"),
            Cdbl => write!(f, "CDBL"),
            Chr => write!(f, "CHR$"),
            Cint => write!(f, "CINT"),
            Cos => write!(f, "COS"),
            Csng => write!(f, "CSNG"),
            Date => write!(f, "DATE$"),
            Exp => write!(f, "EXP"),
            Fix => write!(f, "FIX"),
            Hex => write!(f, "HEX"),
            Inkey => write!(f, "INKEY"),
            Instr => write!(f, "INSTR"),
            Int => write!(f, "INT"),
            Left => write!(f, "LEFT$"),
            Len => write!(f, "LEN"),
            Log => write!(f, "LOG"),
            Mid => write!(f, "MID$"),
            Oct => write!(f, "OCT"),
            Pos => write!(f, "POS"),
            Right => write!(f, "RIGHT$"),
            Rnd => write!(f, "RND"),
            Sgn => write!(f, "SGN"),
            Sin => write!(f, "SIN"),
            Spc => write!(f, "SPC"),
            Sqr => write!(f, "SQR"),
            Str => write!(f, "STR"),
            String => write!(f, "STRING"),
            Tab => write!(f, "TAB"),
            Tan => write!(f, "TAN"),
            Time => write!(f, "TIME$"),
            Val => write!(f, "VAL"),
        }
    }
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

// New compiled type for loop (to before from)
--eval STEP
--eval TO
--eval FROM
Pop("A")
Literal("A")
Literal(0) // signal start of loop (don't step)
:loop
For(:done) // pop [int],var,to,step; if done goto label ; else push back without int
-- stuff
Goto(:loop)
:done

// Old school for-next
--eval FROM
Pop("A")
--eval TO
--eval STEP
Literal("A")
Literal(:foo)
:foo
...
Next() pop :foo,var,step,to, if !done push all jump foo

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

//if x then stuff
-- eval x
IfNot(:a)
stuff
:a

//if x then a=5 else b=6
-- eval x
IfNot(:else)
--exec a=5
GoTo(:finish)
:else
--exec b=6
:finish

// new input
push return addr
lit(prompt)
lit(#caps)
lit(#len)
Input(var) // pushes stuff+addr, checks len, pushes answers
array evals
pop var
array evals
pop var
Input(nil)


on x gosub

push :return-addr
push len 3
push var
On
goto 1
goto 2
goto 3
:return-addr

*/
