/*!
# `ON expression <GOTO|GOSUB> <line>[,<line>...]`

## Purpose
Branches to a line based on the value of expression.

## Remarks
The value 1 goes to the first line, 2 the second, etc.
Values of 0 or greater than the number of lines do not branch.
Values < 0 cause an `?ILLEGAL FUNCTION CALL` error.

## Example
```text
```

*/
