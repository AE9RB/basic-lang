/*!
# `DIM <variable name>(<dimensions>)[,...]`

## Purpose
Prepare an array by defining its dimensions and size.

## Remarks
Arrays are sparse. You can, for example, define a 32767 by 32767 array
of 1073676300 elements. The number of values that can be stored is 64K
for all variables combined. Index values are integers in the range of
0-32767. Accessing an array before it is dimensioned automatically defines
it with a dimension of 10. The index is inclusive so `DIM X(10)`
allows the use of `X(0)` to `X(10)`

## Example
```text
10 DIM A$(100), X(10,10)
20 A$(42)="THE ANSWER"
30 X(4,2)=2.7182818
40 PRINT A$(42)+"!", X(4,2)
RUN
THE ANSWER!    2.7182817
```
*/
