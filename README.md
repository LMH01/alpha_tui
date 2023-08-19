# rust_alpha

This is my attempt at writing a compiler for the Alpha-Notation used in my Systemnahe Informatik lecture at university.

## Current status

Internal handling of instructions is finished, programs can be run, when assembled by creating a list of instructions in code. Now the "compiler" needs be be written.

## TODO

- [ ] Make it possible to customize the available memory cells (will be done by cli option)
- [ ] Add option to load predetermined values into memory cells before program starts (cli option, probably read from file)
- [X] Make progamm work with GUI 
	- [ ] (Customization of available accumulators) - will be done with cli options
	- [ ] (Customization of available memory cells) - will be done with cli options
		- if no memory cells are set all commands that require memory cells should be disabled ("compiling" with those commands included should fail)	
- [X] Debug mode -> Step through each instruction
- [X] Add tests (at least one for each command)
- [X] Add support for comments at end of line (marked with # or //)
- [ ] Add log output window to tui and make messages get printed in there
- [ ] Fix instruction pointer when lines are commented out (remove full line comments from list?)

### Instructions

Be $c,u,v\in\mathbb{Z};n\in\mathbb{N}|n\geq0:i,j\in\lbrace h_0,\ldots,h_n\rbrace;op\in\lbrace +,-,*,/\rbrace;cmp\in\lbrace <,\leq,=,\geq,>\rbrace$

Currently the following commands are supported (booth at runtime and when parsed):

- $\alpha_u:=\alpha_v$
- $\alpha_u:=\rho(i)$
- $\alpha_u:=c$
- $\alpha_u:=\alpha_u\space\textbf{op}\space c$
- $\alpha_u:=\alpha_u\space\textbf{op}\space\alpha_v$
- $\alpha_u:=\alpha_v\space\textbf{op}\space\alpha_w$
- $\alpha_u:=\alpha_u\space\textbf{op}\space \rho(i)$
- $\alpha_u:=\rho(i)\space\textbf{op}\space \rho(j)$
- $\rho(i):=\alpha_u$
- $\rho(i):=c$
- $\rho(i):=\rho(j)\space\textbf{op}\space c$
- $\rho(i):=\rho(j)\space\textbf{op}\space\alpha_u$
- $\rho(i):=\rho(j)\space\textbf{op}\space\rho(k)$
- $\rho(i):=\rho(j)$
- if $\alpha_u\space\textbf{cmp}\space\alpha_v$ then goto label
- if $\alpha_u\space\textbf{cmp}\space c$ then goto label
- if $\alpha_u\space\textbf{cmp}\space\rho(i)$ then goto label
- goto label 
- push 
- pop

#### Unofficial instructions

The following instructions are not "official", I added them because I thought that it would be useful:
- $print(\alpha_u)$
- $print(\rho(i))$
- $print("STRING")$ // Prints the contents between the " into the console.
- $println("STRING");$ // Prints the contents between the " into the console. Prints a newline.
