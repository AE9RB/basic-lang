/*!
# Expressions and Types

64K BASIC supports four types of data. This data is stored in a variable.
Variables are simply names that refer to a data value. Variable names
consist of ASCII alphabetic characters followed by optional ASCII numeric
characters. No special characters, such as underbars (_), are valid.

We can assign a value to a variable with the `LET` statement. `LET` has a
shortcut which is that using the word `LET` is optional. If you are familiar
with other languages, it may look like an assignment operation without the `LET`
but there is technically no assignment operator in BASIC.

```text
LET PI = 3.14
PI2 = 6.28
```

You specify one of the four types by optionally decorating the variable or
value with "!", "#", "%", and "$".

```text
LET A = 1.5 ' 32-bit floating point, aka Single
LET A! = 1.5 ' 32-bit floating point, aka Single
LET A# = 1.5 ' 64-bit floating point, aka Double
LET A% = 5   ' Signed 16-bit integer
LET A$ = "X" ' String of up to 255 characters
```

Literals are values included in your source code. For example, "1.5" is a literal.
Literals numbers may be typed using the "!", "#", and "%" decorators.

```text
PRINT 3# + 0.14#
```

If you don't decorate a number, it will be assigned an appropriate type.

 * If the number contains an exponent with the letter E, it is a Single.
 * If the number contains an exponent with the letter D, it is a Double.
 * If the number contains a decimal, it is a Single unless more than 7 digits.
 * If the number has more than 7 digits, it is a Double.
 * If the number fits into an Integer (-32767 to 32767), it is an Integer.

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

Values can only be demoted by assignment with `LET`. If the value won't
fit into an integer variable then you get an `?OVERFLOW` error.
Storing a Double value in a Single variable will result in a loss
of precision.

```text
A% = 300*300         ' ?OVERFLOW
A% = 300+300         ' 600
A! = 1.2345678912345 ' 1.2345679
```

64K BASIC supports the following operators, listed in order of precedence.

| Operators | Meaning |
|-|-|
| - + | Unary negation and unity |
| * / | Multiplication and division |
| \   | Integer division |
| %   | Remainder (aka Modulo) |
| + - | Addition and subtraction |
| = <> < <= > >= | Comparison |
| AND | Bitwise and |
| OR | Bitwise or |
| XOR | Bitwise exclusive or |
| IMP | Bitwise imp |
| EQV | Bitwise eqv |

Integer division is another quirk of BASIC. Dividing two Integers will
promote both Integers to Singles first. Integer division is performed
on the Integers and may result in a `?DIVIDE BY ZERO` or `?OVERFLOW`
errors. Only regular division has this quirky behavior.

Arrays are an extension of variables. You can make single dimension arrays
(vectors) or multi dimensional arrays (matrices). Arrays are dimensioned
before they are used. If you use an array before it is dimensioned, it will be
automatically set to a single dimension of 10 values. Dimensions must be
valid Integers (0 to 32767). A dimension of 10 is actually 11 values (0 to 10)
and a dimension of 0 is a single value numbered 0.

```text
10 DIM BOARD(10,10)
20 LET BOARD(5,5) = 12
```

Arrays are sparse in 64K BASIC. This means you can dimension 32767 values but
none use any memory until you set them to something other than default (0 or "").
Some BASIC implementations would let you set whether arrays start numbering at
0 or 1. Sparse arrays make this irrelevant so arrays always start at 0.

There is a limit of 64K variables. There is no "byte limit". You can have 64K
variables of 255 character strings if you want. You won't find any old programs
that come anywhere near this limit but it's something to consider if you
are writing portable BASIC meant to run on old hardware or emulators.

Let's finish this chapter with more complex expressions and a built-in function.
Functions are documented in another chapter.

```text
10 PI = 3.14159
20 PRINT COS(PI*2)
30 PRINT (100 + PI) * 9.695411
```

*/
