/*!
# Conversions and Compatibility

64K BASIC aims for compatibility with a time when BASIC usually came on ROM.
Instead of keeping your compiler up to date as we do today, you would adjust
the source code. Here's some tips.

## RANDOMIZE X
Old computers often didn't have useful entropy, not even a real-time clock.
Many implementations would reset the random number generator to the same state
on ever run, but almost all would reset to the same state every boot.
`RANDOMIZE` was used to get around this by asking the user for a seed.
64K BASIC reseeds the random number generator with good entropy on every run,
so in most cases you simply delete the unneeded code.

`RANDOMIZE` without an X will prompt the user for a seed value.
This allowed someone to replay the same game if they wanted.
To emulate this behavior, here's a replacement.

```text
INPUT "Random Number Seed";A:RND=RND(-ABS(A))
```

Probably the most common way to seed the random number generator is to time
how long it takes the user to respond to a prompt. This is not necessary
in 64K BASIC so if you see something similar to the following, you can
delete it.

```text
PRINT "Press any key to continue":WHILE INKEY$="":RND=RND():WEND
```

## GET A$

To get a single keypress and store it in A$, 64K BASIC uses the newer style:
```text
A$ = INKEY$
```

## Integer BASIC
Some versions of BASIC didn't support floating point types. Use `DEFINT`
to change the default variable type to integer. The RND function needed
to work without floats so we'll `DEF FN` a new one. Replace every `RND`
with `FNRND`.
```text
10 DEFINT A-Z:DEF FNRND(X)=INT(RND()*X+1)
```

## LOCATE ROW,COL

This is probably a GW-BASIC program so it might use graphics and sound.
However, in many cases it is simply used to center text on the screen.
Use `STRING$` to get a bunch of newlines and `SPC(X)` to move to the column.
```text
REM CLS:LOCATE 5,20:PRINT "TITLE"
CLS:PRINT STRING$(5,10)SPC(20)"TITLE"
```

## SOUND and BEEP

64K BASIC does not support sound. Your terminal might beep with:
```text
PRINT CHR$(7)
```

## KEY OFF/ON

This controls display of the status line in GW-BASIC. Usually a program turns
it off at the start and back on at the end. You can delete these.

## OPTION BASE

This selects if arrays start at 0 or 1. This isn't needed since memory isn't scarce.
Also, 64K BASIC arrays are sparse so by simply not using 0 it won't be allocated.

*/
