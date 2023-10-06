[![Downloads](https://img.shields.io/github/v/release/lmh01/alpha_tui)](https://github.com/lmh01/alpha_tui/releases)
![Downloads](https://img.shields.io/github/downloads/lmh01/alpha_tui/total)
[![License](https://img.shields.io/github/license/lmh01/alpha_tui)](LICENSE)

# alpha_tui

This is my attempt at writing a compiler for the Alpha-Notation used in my Systemnahe Informatik lecture at university.

Programs are read in and then compiled, a terminal ui is then opened where you can run the program line by line or by using breakpoints.

Pull requests and bug reports are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for further details.

## Getting started

1. Download the [newest release](https://github.com/lmh01/alpha_tui/releases/latest) specific for your system
2. Extract the .zip file 
3. Create a program by using the text editor of your choice or try an example program located in [examples/programs](examples/programs). The examples might help you write your program.
4. Run `alpha_tui` by opening a terminal in the folder and then typing `alpha_tui load FILENAME`, for an example program this command could look like this: `alpha_tui load examples/programs/faculty.alpha`
5. The terminal ui will open where you can run the program line by line by using the `[r]` key

### Compile from source

To compile the program from source the rust toolchain is needed. Once installed you can run the program by typing `cargo run`. To submit arguments you can use `--`, for example `cargo run -- -h` will print help.

## Instructions

See [instructions](docs/instructions.md).

You can use either `#` or `//` to mark inline or full-line comments.

## CLI Commands and options

See [cli](docs/cli.md) or `alpha_tui help`.

## Interface and usage

The interface is written using the [ratatui](https://github.com/ratatui-org/ratatui) library.

When a program is opened it can look like this: ![Program loaded example](media/gui_program_loaded.png)

Press `[r]` to begin to run the program, subsequent instructions can also be run with `[r]`. Values that have changed and the line that was run last are highlighted.  This can look like this: ![Program running example](media/gui_program_running.png)

When the last instruction was executed the following window is displayed. You can restart by pressing `[s]` or exit the program by pressing `[q]`. ![Program finished example](media/gui_program_finished.png)

### Debug features

Some debug features require you to select a line in which a debug action should take place.

You can enter debug select mode by pressing `[d]`, this could look like this: ![Debug select mode](media/gui_debug_select_mode.png)

Navigate by using the `arrow keys`, for ease of use `[w]` and `[s]` are also supported.

#### Breakpoints

Breakpoints can be set to run all lines of code up until the line in which the breakpoint is set.

To set a breakpoint enter `debug select mode` and press `[t]` in the line in which you want to set the breakpoint. A star to the left will indicate where a breakpoint is placed.

A placed breakpoint can look like this: ![Breakpoint set](media/gui_breakpoint_set.png)

Press `[n]` when in run mode to make the program run automatically to the next breakpoint (note how the values to the right have been updated): ![Next breakpoint](media/gui_breakpoint_mode_run.png)

#### Jump to line

When in `debug select mode` you can select a line and jump directly to it using `[j]`, skipping all other instructions. You should however be careful when using this, because runtime errors are far more likely to occur due to uninitialized accumulators or memory cells.
Functions may also no longer be properly exited because of a misaligned call stack.

### Error handling

[Miette](https://github.com/zkat/miette) is used for error handling, this provides helpful error messages when a program can not be compiled due to an unknown instruction.

Such error could look like this ![Miette error handling](media/miette_error.png)

or this: 

![Miette error handling](media/miette_error_2.png)

## Future ideas

- [ ] Make instruction list scroll down to make 3 instructions before the current one always displayed
    - ratatui currently does not provide a simple solution for this
- [ ] Add command line parameter that allows a program to be run where the content of a specific accumulator or memory cell is compared against a defined value that is provided when the program is launched. Alpha_tui will exit with 0 if the resulting value is equal to the provided value. This will make it possible to automate tests for alpha notation programs.
    - This can be implemented using the check subcommand
- [ ] Move backend (internal runtime environment) into own project which makes it possible to write new programs without the need to copy the backend of this program
- [ ] Text editor inside the program to write new alpha notation programs
    - This would however come with a drawback, the nice error messages could probably not be shown inside the tui.
