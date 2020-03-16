/*!
# `FOR <variable>=x TO y [STEP z]`
Where x, y, and z are expressions.
## Purpose
Used with `NEXT` to repeat execution of statements
while iterating over a sequence of numbers.

## Remarks
If we wanted the numbers 1,3,5,7 we would write `FOR I=1 TO 7 STEP 2`.
On the first iteration, 1 will be assigned to variable I.
Statements execute until a `NEXT` statement.
On subsequent iterations, the variable I gets 2 added to it.
If the result exceeds 7 the loop breaks.
Otherwise the statements get executed again.

The first iteration will evaluate z, then y, then set variable to x.
Note that some versions of BASIC evaluate x then y then z; this is the older style.
The evaluated x and y along with the variable name is stored on the stack.
Because `FOR` loops are stack-based, you can nest them for iterating over
multiple dimensions.

The expressions are skipped if the variable is already past the end
on the first execution. Some versions of basic always execute
the statements because the logic is done at the NEXT statement.

## Example 1
```text
10 I=9
20 FOR I=1 TO 10 STEP I
30 PRINT "HELLO WORLD";i
40 NEXT I
RUN
HELLO WORLD 1
HELLO WORLD 10
```

## Example 2
```text
10 FOR X=1 TO 2
20 FOR Y=5 TO 6
30 PRINT x,y
40 NEXT Y,X
RUN
 1  5
 1  6
 2  5
 2  6
```

*/
