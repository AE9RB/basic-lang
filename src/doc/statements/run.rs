/*!
# `RUN [<line number> | <filename>]`

## Purpose
Clear memory and start the program.

## Remarks
This is a shortcut for `CLEAR:GOTO <line number>`.
Omitting the line number defaults to the first line.
Providing a filename loads a program and runs it.

## Example
```text
10 PRINT "Hello World"
RUN
Hello World
```

*/
