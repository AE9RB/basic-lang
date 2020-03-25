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

The first iteration will evaluate x, then y, then z.
Newer versions of BASIC evaluate z first.

The first iteration always executes even if starting past the end.
Newer versions of BASIC may skip the first iteration.

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
