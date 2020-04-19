/*!
# Limits and Internals

ROM BASIC was interpreted from slightly tokenized strings. Lexical
analysis was a simple scan for keywords which would be collapsed
to the unprintable ASCII characters below 32. These tokenized
strings would be parsed as they were executed. There was no
syntax tree or any opportunities for optimization. BASIC was
notorious for being slow.

64K BASIC is a compiler. Lexical analysis is crippled to mimic
ROM BASIC but after that it parses to a nice abstract syntax tree.
The syntax tree is compiled into link objects. The link objects
contain opcodes which are resolved into a program for a virtual
machine. The virtual machine is custom for 64K BASIC.

All compilation happens behind the scenes. Big programs compile
in a few milliseconds so you won't notice any delay. Direct mode
opcodes are appended to the end of the indirect mode opcodes.
The program is not compiled for every direct mode statement, instead
the opcodes from previous direct mode statements are dropped and
the new opcodes are linked to the end of the existing program.

Each of the following is its own independent memory pool. Since
ROM BASIC typically has about 40K bytes for everything, old programs
will never come close to a limit in 64K BASIC.

Source code is UTF-8. There is a maximum of 65530 lines (0-65529).
Each line is limited to 1024 characters (not bytes).

Compiled code is limited to 64K instructions (not bytes).
This is what the virtual machine executes.

Data is limited to 64K values (not bytes).
These are from `DATA` statements.

Stack is limited to 64K values (not bytes). In addition to the usual
values like numbers and strings, the stack contains information about
`FOR` loops and `GOSUB` calls.

Variables are limited to 64K allocated values (not bytes). Zeros and
empty strings are not allocated. Arrays are variables and therefore
part of this 64K pool.

Now you know why it's called 64K BASIC.

*/
