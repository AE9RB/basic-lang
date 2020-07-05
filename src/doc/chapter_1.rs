/*!
# Expressions and Types

64K BASIC supports four types of data. This data is stored in a variable.
Variables are simply names that refer to a data value. Variable names
consist of ASCII alphabetic characters followed by optional ASCII numeric
characters. No special characters, such as underbars (_), are valid.

A value is assigned to a variable with the `LET` statement. `LET` has a
shortcut which is that using the word `LET` is optional. If you are familiar
with other languages, it may look like an assignment operation without the `LET`
but there is technically no assignment operator in BASIC.

```text
LET PI = 3.14
PI2 = 6.28
```

There are three numeric types and the string type. Decorating the variable
with "!", "#", "%", and "$" will explicitly request a type.

```text
LET A! = 1.5 ' Single, 32-bit floating point
LET A# = 1.5 ' Double, 64-bit floating point
LET A% = 5   ' Integer, signed 16-bit
LET A$ = "X" ' String of up to 255 characters
LET A = 1.5  ' Single unless changed with `DEFINT/SNG/DBL/STR`
```

Literals are unchanging values included in your source code. For example, "1.5" is
a literal. Literals numbers may be typed using the "!", "#", and "%" decorators.

```text
PRINT 3# + 0.14#
```

If you don't decorate a literal number, it will be assigned an appropriate type.

 * If the number contains an exponent with the letter E, it is a Single.
 * If the number contains an exponent with the letter D, it is a Double.
 * If the number contains a decimal, it is a Single unless more than 7 digits.
 * If the number has more than 7 digits, it is a Double.
 * If the number fits into an Integer (-32767 to 32767), it is an Integer.
 * Anything that doesn't match the above is a Single.

You can't have -32768 as a literal Integer although you can store -32768
as the result of an expression into an Integer variable. This is one of
many quirks of BASIC that 64K BASIC preserves.

Integers may also be specified in hexadecimal or octal with the "&" decorator.

```text
&10  ' Octal for 8
&010 ' Octal for 8
&H0D ' Hex for 13
```

Values are promoted as needed to preserve precision. For example, adding
a Single to a Double causes the Single to be promoted and the addition
done on two Doubles with the result being a Double.

Values are automatically demoted only by assignment with `LET`. If the value
won't fit into an integer variable then you get an `?OVERFLOW` error.
Storing a Double value in a Single variable will result in a loss
of precision.

```text
A% = 300*300         ' ?OVERFLOW
A% = 300+300         ' 600
A! = 1.2345678912345 ' 1.2345679
```

Strings may contain up to 255 characters. These are unicode characters, not graphemes
or bytes. String literals are surrounded by quotation marks. There is no escape
sequence so getting quotation marks in your string requires the use of a function.
Source files are UTF-8 but only strings and comments may contain non-ASCII.

```text
A1$ = "Hello"
A2$ = CHR$(34) + "HELLO" + CHR$(34)
```

Expressions are anything that evaluates to a value. The number `1` is an expression;
literals are a specific kind of expression. The variable `PI#` is also an expression.
An expression may also perform arithmetic, compare values, and call functions.
Here are some example of expressions.

```text
A + PI
2 / (A + B)
CHR$(34)
```

64K BASIC supports the following operators, listed in order of precedence.

| Precedence | Operators | Meaning |
|-|-|-|
| 13 | ^ | Raise to a power |
| 12 | - + | Unary negation and unity |
| 11 | * / | Multiplication and division |
| 10 | \   | Integer division |
| 9 | %   | Remainder (aka Modulo) |
| 8 | + - | Addition and subtraction |
| 7 | = <> < <= > >= | Relational |
| 6 | NOT | Bitwise not, unary |
| 5 | AND | Bitwise and |
| 4 | OR | Bitwise or |
| 3 | XOR | Bitwise exclusive or |
| 2 | IMP | Bitwise imp |
| 1 | EQV | Bitwise eqv |

Integer division (`\`) is a quirk of BASIC. Regular division of two Integers
will promote both Integers to Singles first. Integer division is performed
only on Integers and may result in a `?DIVIDE BY ZERO` or `?OVERFLOW`
error. Only regular division has this quirky promotion behavior to maintain
compatibility with early versions of BASIC that don't have an Integer type.
All Integer arithmetic is always checked for overflows and division by zero.

```text
PRINT 10000*3 ' 30000
PRINT 10000*4 ' ?OVERFLOW
PRINT 10/3     ' 3.3333333
PRINT 10\3     ' 3
PRINT 10/0     ' inf
PRINT 10\0     ' ?DIVISION BY ZERO
```

Relational operators always return an Integer with a value of 0 for true
or -1 (&xFFFF) for false. All relational operators evaluate at the same
precedence. These are typically used with IF statements, but there are
other uses if you take advantage of the 0 and -1 value guarantee.

| Operator | Meaning |
|-|-|
| = | Equality |
| <> | Inequality |
| < | Less than |
| <= | Less than or equal |
| > | Greater than |
| >= | Greater than or equal |

```text
IF 10 < 100 THEN PRINT "INDEED"
((A<B)*1)+((A>B)*-1)
```

Logical operators perform bit-level arithmetic on Integers. Each of the 16 bits are
computed with these truth tables.

<pre><code><u>    X    NOT X </u>      <u> X  Y    X AND Y </u>     <u> X  Y    X OR Y </u>
    1      0          1  1       1          1  1       1
    0      1          1  0       0          1  0       1
                      0  1       0          0  1       1
                      0  0       0          0  0       0

<u> X  Y    X XOR Y</u>     <u> X  Y    X IMP Y </u>     <u> X  Y    X EQV Y </u>
 1  1       0         1  1       1          1  1       1
 1  0       1         1  0       0          1  0       0
 0  1       1         0  1       1          0  1       0
 0  0       0         0  0       1          0  0       1
</code></pre>

Arrays are an extension of variables. You can make single dimension arrays
(vectors) or multi dimensional arrays (matrices). Arrays are dimensioned
before they are used. If you use an array before it is explicitly dimensioned,
it will be automatically set to a dimension of 10. Dimensions must be positive
Integers (0 to 32767). A dimension of 10 is actually 11 values (0 to 10)
and a dimension of 0 is a single value numbered 0.

```text
10 DIM BOARD(10,10)
20 LET BOARD(5,5) = 12
```

Arrays are sparse in 64K BASIC. This means you can dimension 32767 values but
none use any memory unless they are set them to something other than default (0 or "").
Some BASIC implementations would let you set whether arrays start numbering at
0 or 1. Sparse arrays make this irrelevant so arrays always start at 0.

The next two chapters of this manual is a reference for statements and functions.
64K BASIC is a comprehensive implementation of early BASIC. Most programs only
used a small subset, sometimes called minimal BASIC, so it's not necessary to
read to the end of this book if all you want to do is port programs.

Here's a program you can decipher with the reference to get a feel for BASIC.
Use CHR$(79) if the emoji isn't supported by your terminal.

```text
10 INPUT ,"What is your name";NAME$
20 IF VAL(LEFT$(TIME$,2)) < 5 OR VAL(LEFT$(TIME$,2)) > 10 GOTO 40
30 PRINT "Good morning, " NAME$ ".":GOTO 50
40 PRINT "Hello, " NAME$ "."
50 INPUT "How many cookies would you like";COOKIES
60 ON COOKIES GOTO 80,90,90
70 PRINT "You can't have" COOKIES "cookies.":GOTO 50
80 PRINT "Here is your cookie: " CHR$(127850):END
90 PRINT "COOKIES: ";
100 FOR I=1 TO COOKIES:PRINT CHR$(127850);:NEXT:PRINT
```

*/
