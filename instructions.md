# Instructions

You can replace $\alpha$ with `a` and $\rho$ with `p` when writing your program to make writing of the program easier.
Indices are written directly after `a`/$\alpha$, however you don't need an indice if you wan't to address accumulator 0. For example you can use either `a0 := 5` or `a := 5` to assign accumulator 0 the value `5`.

By jumping to the labels `END`, `ENDE`, `end` or `ende` you can end your program. Example: `goto END`

The following instructions are supported:

Be $c\in\mathbb{Z};n,o,p\in\mathbb{N}|n\geq0:j,k,l\in\lbrace h_0,\ldots,h_n\rbrace;T\in\lbrace\alpha_n, \alpha_o, \alpha_p, \rho(j),\rho(k),\rho(l)\rbrace;S\in\lbrace T, c\rbrace;\textbf{op}\in\lbrace +,-,*,/,\%\rbrace;\textbf{cmp}\in\lbrace <,\leq,=, \ne,\geq,>\rbrace;$

| Formal instruction | code example | explanation |
| - | - | - |
|$T := S $| a0 := p(h1) | |
|$T := S\space\textbf{OP}\space S$ |p(h1) := a0 + 5 | |
|if $S\space\textbf{cmp}\space S$ then goto label| if a0 == a1 then goto loop | |
|goto label | goto loop | if the comparison succeeds the next instruction pointer is updated to the instruction at label|
|stack $\textbf{OP}$ | stack+ | uses the top most values to calculate a new value which is then pushed onto the stack, note that the top most value is the right part of the calculation|
|push | push | pushes the current value of $\alpha_0$/a0 on the stack |
|pop | pop | pops the top value of the stack into $\alpha_0$/a0 |

## Examples

Example programs can be found [here](examples/programs/).

| code | explanation |
| - | - |
| a0 := 5 | loads value $5$ into $\alpha_0$ |
| p(h1) := 10 | loads value $10$ into $\rho(h1)$ |
| a1 := 5 * p(h1) | calculates $5 * \rho(h1)$ and puts the result in $\alpha_1$ |
| if p(h1) != 5 then goto loop | If $\rho(h1)$ is not equal to $5$ then the instruction pointer is updated to the instruction at label $loop$ |

## Comparisons

| Formal | code example |
| - | - |
| $<$ | < |
| $\le$ | <= |
| $=$ | == |
| $\ne$ | != |
| $\ge$ | >= |
| > | > |