# CLI Commands and options

For a full list of options and more explanation see `alpha_tui help`, `alpha_tui run help` or `alpha_tui check help`.

## General options

Accumulators and memory cells are automatically created when the input program is read.
To circumvent that you can set the option `--disable-memory-detection`. You then need to specify the accumulators and memory_cells that should be created. The options `-a`, `-m` and `--memory-cell-file` can be used to specify those values. `--memory-cell-file` can also be used to specify available memory cells, when automatic detection is disabled.

If you require memory cells to be pre initialized you can use the option `--memory-cell-file` to read in a file that contains memory cell information. An example for such file can be found [here](examples/memory_cells.cells).

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