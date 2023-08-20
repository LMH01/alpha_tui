# Instructions

To use these instructions in your program replace $\alpha$ with `a` and $\rho$ with `p`.
Indices are written directly after `a` for example you can use `a0 := 5` to assign accumulator 0 the value `5`.

By jumping to the labels `END`, `ENDE`, `end` or `ende` you can end your program. Example: `goto END`

Example programs can be found [here](examples/programs/).

The following instructions are supported:

Be $c,u,v\in\mathbb{Z};n\in\mathbb{N}|n\geq0:i,j\in\lbrace h_0,\ldots,h_n\rbrace;op\in\lbrace +,-,*,/\rbrace;cmp\in\lbrace <,\leq,=, \ne,\geq,>\rbrace$
| Formal instruction | code example | explanation |
| - | - | - |
|$\alpha_u:=\alpha_v$| a0 := a1 | |
|$\alpha_u:=\rho(i)$| a0 := p(h1) | |
|$\alpha_u:=c$| a0 = 5 | |
|$\alpha_u:=\alpha_u\space\textbf{op}\space c$| a0 := a0 + 5  | |
|$\alpha_u:=\alpha_u\space\textbf{op}\space\alpha_v$| a0 := a0 - a1 | |
|$\alpha_u:=\alpha_v\space\textbf{op}\space\alpha_w$| a0 := a1 * a2 | |
|$\alpha_u:=\alpha_u\space\textbf{op}\space \rho(i)$| a0 := a0 / p(h1) | |
|$\alpha_u:=\rho(i)\space\textbf{op}\space \rho(j)$| a0 := p(h1) + p(h2) | |
|$\rho(i):=\alpha_u$| p(h1) := a0 | |
|$\rho(i):=c$| p(h1) := 5 | |
|$\rho(i):=\rho(j)\space\textbf{op}\space c$| p(h1) := p(h2) - 5 | |
|$\rho(i):=\rho(j)\space\textbf{op}\space\alpha_u$| p(h1) := p(h2) * a0 | |
|$\rho(i):=\rho(j)\space\textbf{op}\space\rho(k)$| p(h1) := p(h2) / p(h3) | |
|$\rho(i):=\rho(j)$| p(h1) := p(h2) | |
|if $\alpha_u\space\textbf{cmp}\space\alpha_v$ then goto label| if a0 == a1 then goto loop | |
|if $\alpha_u\space\textbf{cmp}\space c$ then goto label| if a0 != 5 then goto loop | |
|if $\alpha_u\space\textbf{cmp}\space\rho(i)$ then goto label| if a0 <= p(h1) then goto loop | |
|goto label | goto loop | |
|push | push | pushes the current value of $\alpha_0$/a0 on the stack |
|pop | pop | pops the top value of the stack into $\alpha_0$/a0 |

## Comparisons

| Formal | code example |
| - | - |
| $<$ | < |
| $\le$ | <= |
| $=$ | == |
| $\ne$ | != |
| $\ge$ | >= |
| > | > |