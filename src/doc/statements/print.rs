/*!
# `PRINT [<list of expressions>]`

## Purpose
Output information to the terminal for the operator.

## Remarks
A `PRINT` by itself outputs a newline (ASCII 10).
To suppress the newline, use a semicolon (;) at the end.
Separating expressions with nothing or a semicolon (;) will print them with nothing between.
Output is divided into zones of 14 characters. A comma will advance to the start of next zone.

## Example
```text
PRINT ,"Mar","Apr":?"Bought",100,120:?"Sold",-97,-123
              Mar           Apr
Bought         100           120
Sold          -97           -123
```

*/
