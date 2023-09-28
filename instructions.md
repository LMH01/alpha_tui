# Instructions

Indices are written directly after `a`/ $\alpha$, however you don't need an indice if you want to address accumulator 0. For example you can use either `a0 := 5`, `a := 5` or $\alpha$ `:= 5` to assign accumulator 0 the value `5`.

By jumping to the labels `END`, `ENDE`, `end` or `ende` you can end your program. Example: `goto END`

You can define a custom start point for your program with the labels `main` or `MAIN`.

The following instructions are supported:

Be $c\in\mathbb{Z};n,o,p\in\mathbb{N}|n\geq0:j,k,l\in\lbrace h_0,\ldots,h_n\rbrace;T\in\lbrace\alpha_n, \alpha_o, \alpha_p, \rho(j),\rho(k),\rho(l)\rbrace;S\in\lbrace T, c\rbrace;\textbf{op}\in\lbrace +,-,\times,\div,\%\rbrace;\textbf{cmp}\in\lbrace <,\leq,=, \ne,\geq,>\rbrace;$

| Formal instruction | code example | explanation |
| - | - | - |
|$T := S $| $\alpha0$ := $\rho(h1)$ | |
|$T := S\space\textbf{OP}\space S$ |$\rho$(h1) := $\alpha 0$ + 5 | |
|if $S\space\textbf{cmp}\space S$ then goto label| if $\alpha 0$ == $\alpha 1$ then goto loop | if the comparison succeeds the next instruction pointer is updated to the instruction at label |
|goto label | goto loop | the next instruction pointer is updated to the instruction at label|
|stack $\textbf{OP}$ | stack+ | uses the top most values to calculate a new value which is then pushed onto the stack, note that the top most value is the right part of the calculation, also works when operand is separated by a space like this: "stack +"|
|push | push | pushes the current value of $\alpha_0$/a0 on the stack |
|pop | pop | pops the top value of the stack into $\alpha_0$/a0 |
|call label | call function | the next instruction pointer is updated to the instruction and a return address is set |
|return | return| returns from the current function to the point where the instruction was called, if return is called inside the main function/without previous function being called, the program exits|

## Substitutions

The following symbols can be substituted to make writing programs easier

| Source | Sub |
| - | - |
| $\alpha$ | a |
| $\rho$ | p |
| $\times$| * |
| $\div$ | / |
| $\le$ | <= |
| $=$| == |
| $\ne$ | != |
| $\ge$ | >= |

## Examples

Example programs can be found [here](examples/programs/).

| code | explanation |
| - | - |
| a0 := 5 | loads value $5$ into $\alpha_0$ |
| p(h1) := 10 | loads value $10$ into $\rho(h1)$ |
| a1 := 5 * p(h1) | calculates $5 * \rho(h1)$ and puts the result in $\alpha_1$ |
| if p(h1) != 5 then goto loop | If $\rho(h1)$ is not equal to $5$ then the instruction pointer is updated to the instruction at label $loop$ |
