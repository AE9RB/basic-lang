/*!
# Statements

Statements describe what the program does. The original Dartmouth BASIC
was separated into a compiler and operating system with statements like `GOTO`
and `PRINT` for the compiled language and commands like `SAVE` and `LIST`
for controlling the system. 64K BASIC is a unique implementation where
everything is compiled and executed by a virtual machine. There are no
commands, everything is a statement.

Statements tend to be short and it's common to put multiple statements on a
single line. Use a colon `:` to separate statements. This works in both
direct and indirect modes.

```text
FOR I = 1 TO 10 : PRINT I : NEXT I
```

Statement words can never be used in variable names. 64K BASIC will insert
spaces to help you when you accidentally include a word in a variable name.

```text
LET BONK = 1
10 LET B ON K = 1
```

Statements need to be properly formatted with the information they need.
Angled brackets `<>` are used to indicate required items.
Square brackets `[]` are used to indicate optional items.
Ellipsis `...` indicate items that may repeat.
Vertical bars `|` separate mutually exclusive options.
All letters and punctuation not in brackets are required.
*/

#[path = "statements/clear.rs"]
#[allow(non_snake_case)]
pub mod CLEAR;

#[path = "statements/cls.rs"]
#[allow(non_snake_case)]
pub mod CLS;

#[path = "statements/cont.rs"]
#[allow(non_snake_case)]
pub mod CONT;

#[path = "statements/data.rs"]
#[allow(non_snake_case)]
pub mod DATA;

#[path = "statements/def.rs"]
#[allow(non_snake_case)]
pub mod DEF;

#[path = "statements/deftype.rs"]
#[allow(non_snake_case)]
pub mod DEFTYPE;

#[path = "statements/delete.rs"]
#[allow(non_snake_case)]
pub mod DELETE;

#[path = "statements/dim.rs"]
#[allow(non_snake_case)]
pub mod DIM;

#[path = "statements/end.rs"]
#[allow(non_snake_case)]
pub mod END;

#[path = "statements/erase.rs"]
#[allow(non_snake_case)]
pub mod ERASE;

#[path = "statements/for.rs"]
#[allow(non_snake_case)]
pub mod FOR;

#[path = "statements/gosub.rs"]
#[allow(non_snake_case)]
pub mod GOSUB;

#[path = "statements/goto.rs"]
#[allow(non_snake_case)]
pub mod GOTO;

#[path = "statements/if.rs"]
#[allow(non_snake_case)]
pub mod IF;

#[path = "statements/input.rs"]
#[allow(non_snake_case)]
pub mod INPUT;

#[path = "statements/let.rs"]
#[allow(non_snake_case)]
pub mod LET;

#[path = "statements/list.rs"]
#[allow(non_snake_case)]
pub mod LIST;

#[path = "statements/load.rs"]
#[allow(non_snake_case)]
pub mod LOAD;

#[path = "statements/mid.rs"]
#[allow(non_snake_case)]
pub mod MID;

#[path = "statements/new.rs"]
#[allow(non_snake_case)]
pub mod NEW;

#[path = "statements/next.rs"]
#[allow(non_snake_case)]
pub mod NEXT;

#[path = "statements/on.rs"]
#[allow(non_snake_case)]
pub mod ON;

#[path = "statements/print.rs"]
#[allow(non_snake_case)]
pub mod PRINT;

#[path = "statements/read.rs"]
#[allow(non_snake_case)]
pub mod READ;

#[path = "statements/rem.rs"]
#[allow(non_snake_case)]
pub mod REM;

#[path = "statements/renum.rs"]
#[allow(non_snake_case)]
pub mod RENUM;

#[path = "statements/restore.rs"]
#[allow(non_snake_case)]
pub mod RESTORE;

#[path = "statements/run.rs"]
#[allow(non_snake_case)]
pub mod RUN;

#[path = "statements/save.rs"]
#[allow(non_snake_case)]
pub mod SAVE;

#[path = "statements/stop.rs"]
#[allow(non_snake_case)]
pub mod STOP;

#[path = "statements/swap.rs"]
#[allow(non_snake_case)]
pub mod SWAP;

#[path = "statements/tron.rs"]
#[allow(non_snake_case)]
pub mod TRON;

#[path = "statements/while.rs"]
#[allow(non_snake_case)]
pub mod WHILE;
