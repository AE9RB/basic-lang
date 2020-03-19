/*!
# `CONT`

## Purpose
Continue running an interrupted program.

## Remarks
Programs may be interrupted with an error condition,
by pressing CTRL-C, or the STOP and END statements.

## Example
```text
10 PRINT "HELLO"
20 END
30 PRINT "WORLD"
RUN
HELLO
CONT
WORLD
```

*/
