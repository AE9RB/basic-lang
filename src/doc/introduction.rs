/*!
# Introductory Tutorial for 64K BASIC

Begin by opening a terminal and running the executable. Double clicking
the executable from a GUI desktop often works as well. If you get the
following, you have achieved success and are ready for this tutorial.
Type CTRL-D top exit 64K BASIC.
<pre><code>&nbsp;  64K BASIC
&nbsp;  READY.
&nbsp;> █
</code></pre>

 Stop a running program with CTRL-C.

64K BASIC is interactive just like it was back in 1964 when the idea of an
ordinary person sitting in front of a terminal and directly interacting
with a computer was revolutionary. A primary design goal for 64K BASIC
is to capture that experience. Except with better error reporting and
a modern input system.

When you see the `READY.` prompt, 64K BASIC is ready to accept a statement.
A statement describes the work you want the computer to do. Let's tell the
computer to print something. For this tutorial, I'll mark lines that you
type with a "`>`". I'll also not include the `READY.` every time after this.
Go ahead and try your first statement. Type in the marked line followed by
ENTER.

<pre><code>&nbsp;  READY.
&nbsp;> print "Hello World"
&nbsp;  Hello World
&nbsp;  READY.
</code></pre>

Entering a statement which executes immediately is called direct mode.
To make more interesting programs, we'll have to assemble many statements
together into a program. Next, we'll put the same statement into a program
by assigning it to a line number. To do this, simply preceed the statement
with any decimal integer between 0 and 65529 inclusive.

<pre><code>&nbsp;> 10 print "Hello World"
</code></pre>

Nothing happens. This is called indirect mode. The statement is saved to
be executed later. Let's try a couple new statements.

<pre><code>&nbsp;> LIST
&nbsp;  10 PRINT "Hello World"
&nbsp;  READY.
&nbsp;> RUN
&nbsp;  Hello World
</code></pre>

Now that we have a program in memory, we can add more lines or edit existing
lines. To edit a line, type the line number and press TAB. The line will
be loaded into the input buffer for you to edit.

<pre><code>&nbsp;> 10<i>{TAB}</i>
&nbsp;> 10 PRINT "Hello World"
</code></pre>

Linux users may have already noticed the input system is based on readline
and even reads your `inputrc` file. Feel free to explore these capabilities,
but for now you only need the basics like TAB, BACKSPACE, and the arrow keys.

You may be working out a problem in direct mode which doesn't succeed on the
first try. You can access a history of direct mode statements with the up/down arrows.

<pre><code>&nbsp;> PAINT "Hello World"
&nbsp;  <b>?SYNTAX ERROR</b>
&nbsp;> <i>{UP}</i>
&nbsp;> PAINT "Hello World"
</code></pre>

You can `SAVE` a program to the filesystem or `LOAD` one that you previously
saved or downloaded. Filenames are specified absolute or relative to the
current directory of your operating system when 64K BASIC was started.
The `NEW` command erases the program in memory.

<pre><code>&nbsp;> 10 print "Hello World
&nbsp;> save "hello.bas
&nbsp;  READY.
&nbsp;> new
&nbsp;  READY.
&nbsp;> list
&nbsp;  READY.
&nbsp;> load "hello.bas
&nbsp;  READY.
&nbsp;> list
&nbsp;> 10 PRINT "Hello World"
</code></pre>

Let's create a multi-line program for the last example of this tutorial.
The program will ask the user for a number, print its square root, and repeat
indefinitely. Because this is an infinite loop, the program will run forever
or until it's interrupted. Typing CTRL-C interrupts a program.

<pre><code>&nbsp;> 10 input "Your number"; a
&nbsp;> 20 print "The square root of" a "is" sqr(a)
&nbsp;> 30 goto 100
&nbsp;> run
&nbsp;  <b>?UNDEFINED LINE IN 30:9</b>
&nbsp;> list 30
&nbsp;> 30 GOTO <u>100</u>
&nbsp;  READY.
&nbsp;> 30 GOTO 10 <i>{Remember to use TAB here}</i>
&nbsp;> run
&nbsp;> Your number? -8
&nbsp;  The square root of-8 is NaN
&nbsp;> Your number? 9
&nbsp;  The square root of 9 is 3
&nbsp;> Your number?<i>{CTRL-C}</i>
&nbsp;  <b>?BREAK IN 10</b>
&nbsp;  READY.
</code></pre>

Line 30 was intentionally wrong to demonstrate a compiler error. 64K BASIC
has two types of errors, compiler errors and runtime errors. A lot more
information is available at compile time which enables underlining the
offending section when listing a program. Runtime errors will only display
the line number.

This concludes the introductory tutorial. The remainder of this manual is
reference material covering everything 64K BASIC can do. There's a lot of
programs which will run without modification, but every implementation of
BASIC has quirks. Appendix A has information about these quirks and
suggestions for converting them to the 64K BASIC dialect.

*/
