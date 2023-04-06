# rust_alpha

This is my try at writing a compiler for the Alpha-Notation used in my Systemnahe Informatik lecture at university.

## Current status

Development has just begun, almost everything is still missing.

## TODO

- [ ] Make it possible to customize the available memory cells (preferably by adding an option to the gui)
- [ ] Make progamm work with GUI 
	- [ ] (Customization of available accumulators)
	- [ ] (Customization of available memory cells)
		- if no memory cells are set all commands that require memory cells should be disabled ("compiling" with those commands included should fail)	
- [ ] Debug mode -> Step through each instruction
- [ ] Add tests (at least one for each command)

### Instructions

Be $k,u,v\in\mathbb{Z};n\in\mathbb{N}|n\geq0:i,j\in\{h_0,\ldots,h_n\};op\in\{+,-,*,/\};cmp\in\{<,\leq,=,\geq,>\}$

#### Text parsing to instruction

This section logs what instructions can be parsed from text

- [ ] $\alpha_u:=\alpha_v$
- [ ] $\alpha_u:=\rho(i)$
- [ ] $\rho(i):=\alpha_u$
- [ ] $\alpha_u:=k$
- [ ] $\alpha_u:=\alpha_u\space\textbf{op}\space \rho(i)$
- [ ] $\alpha_u:=\rho(i)\space\textbf{op}\space \rho(j)$
- [ ] $\rho(i):=\rho(j)\space\textbf{op}\space\alpha_u$
- [ ] $\rho(i):=\rho(j)$
- [ ] if $\alpha_u\space\textbf{cmp}\space0$ then goto label
- [ ] goto label 
- [ ] push 
- [ ] pop

#### Internal handling of instructions 

This section logs what instructions are implemented in the backend

- [ ] $\alpha_u:=\alpha_v$
- [X] $\alpha_u:=\rho(i)$
- [X] $\rho(i):=\alpha_u$
- [X] $\alpha_u:=k$
- [ ] $\alpha_u:=\alpha_u\space\textbf{op}\space \rho(i)$
- [ ] $\alpha_u:=\rho(i)\space\textbf{op}\space \rho(j)$
- [ ] $\rho(i):=\rho(j)\space\textbf{op}\space\alpha_u$
- [ ] $\rho(i):=\rho(j)$
- [ ] if $\alpha_u\space\textbf{cmp}\space0$ then goto label
- [ ] goto label 
- [X] push 
- [X] pop
