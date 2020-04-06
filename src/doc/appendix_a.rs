/*!
# Conversions and Compatibility

64K BASIC is the classic experience of 8-bit BASIC on a modern terminal.
Part of that experience was a slightly different version of BASIC for
each model of computer. BASIC was usually on ROM which wasn't trivial to
update like modern storage, so you were stuck with the same version for
the entire life of the computer. So instead of keeping your compiler up
to date as we do today, you would know how to adjust programs written
on another platform. Most books with program listings would include a page
or two about the dialect the code was written for. This is that page.

## `RANDOMIZE X`
The random number generator in BASIC will start in the same state every
boot of the computer. Some implementations will even reset it every run.
64K BASIC uses the `RND()` function with a negative number as a seed
instead of the `RANDOMIZE` statement.

`RANDOMIZE` without an X will prompt the user for a seed value.
Use this as a replacement:

```text
INPUT "Random Number Seed";A:RND=RND(-ABS(A))
```

Probably the most common way to seed the random number generator is to time
how long it takes the user to respond to a prompt. Here's how to do that
in 64K BASIC:

```text
PRINT "Press any key to continue";:WHILE INKEY$="":RND=RND():WEND:PRINT
```

## `GET A$`

To get a single keypress and store it in A$, 64K BASIC uses the newer style:
```text
A$ = INKEY$
```

## `LOCATE ROW,COL`

This is probably a GW-BASIC program so it might use graphics and sound.
However, in many cases it is simply used to center text on the screen.
Use a `PRINT` statement for each row you want to move down and `TAB(-X)`
to move to the column.
```text
REM CLS:LOCATE 3,20:PRINT "TITLE"
CLS:PRINT:PRINT:PRINT TAB(-20) "TITLE"
```

## `SOUND` and `BEEP`

64K BASIC does not support sound. Your terminal might with:
```text
PRINT CHR$(7)
```

*/
