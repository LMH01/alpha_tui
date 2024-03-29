# CLI Commands and options

For a full list of options and more explanation see `alpha_tui help`, `alpha_tui load help` or `alpha_tui check help`.

## General options

Accumulators and memory cells are automatically created when the input program is read.
To circumvent that you can set the option `--disable-memory-detection`. You then need to specify the accumulators and memory_cells that should be created. The options `-a`, `-m` and `--memory-cell-file` can be used to specify those values. `--memory-cell-file` can also be used to specify available memory cells, when automatic detection is disabled.

If you require memory cells to be pre initialized you can use the option `--memory-cell-file` to read in a file that contains memory cell information. An example for such file can be found [here](../examples/memory_cells.cells).

### Allowed instructions

You can use the option `--allowed-instructions` to specify a file where allowed instructions are stored. When this option is provided, all programs will fail to build that contain instructions that are not included in the file.

This makes it possible to challenge yourself into working with only a limited instruction set.

#### How it works

All commands that are understood by the program can be used to specify what instructions should be allowed, in addition to that there are some shortcuts available:

| Shortcut | Explanation |
| - | - |
| A | any accumulator |
| M | any memory cell |
| C | any constant value |
| Y | gamma accumulator |
| OP | any operation |
| CMP | any comparison |

Furthermore it is not required to specify a label for the following instructions: `goto, call, if _ then goto`.

This results in this file

```
A := C
M := A OP C
A := M OP C
M := Y
push
pop
stackOP
call
goto
if A CMP M then goto
```
or this file

```
a := 5
p(h) := a OP 3
a := p(h) OP 3
p(h) := y
push
pop
stack+
call label
goto label
if a == p(h) then goto label
```

allowing instructions like these

```
a0 := 5
p(h1) := a0 * 4
a0 := p(h1) + 2
p(h2) := y
push
pop
stack*
call label
goto label
if a == p(h) then goto label
```
to be used in the program.

**It is important to understand that only the type of instruction is limited by this option, to specifically limit what memory locations or what comparisons/operations are available you can use the options `-a`, `-m` or `--memory-cell-file` and `--allowed-comparisons` and `--allowed-operations` . This means that even though you might write `p(h1)` in the allowed instructions file, all available memory cells are allowed in this position, not just `p(h1)`!**

An example file can be found here: [examples/allowed_instructions.txt](../examples/allowed_instructions.txt);

### Allowed comparisons

You can use the option `--allowed-comparisons` to specify all comparisons that should be allowed. If this option is not set, all comparisons are allowed.

The comparisons that can be specified are:

| Comparison | Token | Meaning |
| - | - | - |
| < | lt | lower than |
| <= | le | lower equal |
| == | eq | equal |
| != | neq | not equal |
| >= | ge | greater equal |
| > | gt | greater than |

For example to only allow equal and not equal comparisons you can use this option: `--allowed-comparisons "eq,neq"`

**Note**: At least one comparison needs to be specified, if you would like to prevent the use of any comparison you can use `--allowed-instructions` to limit the available instructions into only allowing instructions which don't take any comparisons.

### Allowed operations

You can use the option `--allowed-operations` to specify all operations that should be allowed. If this option is not set, all operations are allowed.

The operations that can be specified are:

| Operation | Token | Meaning |
| - | - | - |
| + | add | addition |
| - | sub | subtraction |
| * | mul | multiplication |
| / | div | division |
| % | mod | modulo |

For example to only allow addition and subtraction you can use this option: `--allowed-operations "add,sub"`

**Note**: At least one operation needs to be specified, if you would like to prevent the use of any operation you can use `--allowed-instructions` to limit the available instructions into only allowing instructions which don't take any operations.

## Check command

The `check` subcommand can be used to perform checks on the program. It is currently only supported to check if the program compiles. For example the command `alpha_tui check examples/programs/faculty.alpha compile` will check if the program compiles and return `0` if it did. Otherwise an error code is returned, see below for the meaning.

### Return values

These are the different return values of the check command:

| value | meaning |
| -: | - |
| 0 | check was successful |
| 1 | compilation error |
| 2 | runtime error |
| 10 | io error |

## Load command

The main command to compile and run a program is the `load` command, it takes the file as first parameter. Example: `alpha_tui load examples/programs/faculty.alpha`.

By default the code that is read will be formatted to be easier to read, this can be disabled by using the `--disable-alignment` flag. If you however would like to write the formatted code into the source file you can use the `--write-alignment` flag.

Predetermined breakpoints can be loaded by using the `--breakpoints` flag, it takes multiple line numbers as parameter. Example: `alpha_tui load examples/programs/faculty.alpha -b 5`.

## Examples

### Maximum limitation

If you would like to limit what instructions, comparisons/operations and memory locations are available you can do something like this:

```
alpha_tui load program.alpha
    --disable-memory-detection      // Disable automatic memory detection
    --accumulators 2                // Enable two accumulators (a0 and a1)
    --memory-cells "h1,h2"          // Enable memory cells h1 and h2
    --allowed-comparisons "eq,neq"  // Allow equal and not equal comparisons
    --allowed-operations "add,sub"  // Allow addition and subtraction operations
    --allowed-instructions          // Allow some specific instructions
```
(The different options have been written each in a new line for better understanding, in the correct command you would write them in one line.)

If run, all programs that don't fulfill these requirements will fail to build.