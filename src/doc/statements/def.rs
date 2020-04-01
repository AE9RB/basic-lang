/*!
# `DEF FN<name>(<argument variables>) = <expression>`

## Purpose
Define a custom user function for use in other expressions.

## Remarks
User functions may call other user functions but circular calls will
result in a stack overflow.

## Example
```text
10 LET PI=3.14159
20 DEF FNDEG(RADIANS)=RADIANS*180/PI
30 PRINT FNDEG(COS(0.707))
RUN
 43.562813
```
*/
