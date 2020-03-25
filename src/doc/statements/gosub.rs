/*!
# `GOSUB <line number>`

## Purpose
Save the program counter on the stack and move execution to the specified line number.

## Remarks
`RETURN` will return execution to the program counter on the stack.

## Example
```text
10 GOSUB 100
20 PRINT "WORLD"
90 END
100 PRINT "HELLO ";
110 RETURN
RUN
HELLO WORLD
```

*/
