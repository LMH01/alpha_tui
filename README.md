# alpha_tui

This is my attempt at writing a compiler for the Alpha-Notation used in my Systemnahe Informatik lecture at university.

Programs are read in and then compiled, a terminal ui is then opened where you can run the program line by line.

## Getting started

1. Download the [newest release]() specific for your system
2. Extract the .zip file 
3. Create a program by using the text editor of your choice or try an example program located in [examples/programs](examples/programs). The examples might help you write your program.
4. Run `alpha_tui` by opening a terminal in the folder and then typing `alpha_tui -i FILENAME`
5. The terminal ui will open where you can run the program line by line by using the `[r]` key

### Compile from source

To compile the program from source the rust toolchain is needed. Once installed you can run the program by typing `cargo run`. To submit arguments you can use `--`, for example `cargo run -- -h` will print help.

## Instructions

See [instructions](instructions.md).

## Options

Accumulators and memory cells are automatically created when the input program is read.
To circumvent that you can set the option `-d`. You then need to specify the accumulators and memory_cells that should be created. The options `-a` and `-m` or `--memory-cell-file` can be used to specify those values.

If you require memory cells to be pre initialized you can use the option `--memory-cell-file` to read in a file that contains memory cell information. An example for such file can be found [here](examples/memory_cells.cells).

For a full list of options and more explanation see `alpha_tui --help`.

## Interface and usage

The interface is written using the [ratatui](https://github.com/ratatui-org/ratatui) library.

When a program is opened it can look like this: ![Program loaded example](media/gui_program_loaded.png)

Press `[r]` to run the next instruction. Values that have changed and the line that was run last are highlighted.  This can look like this: ![Program running example](media/gui_program_running.png)

When the last instruction was executed the following window is displayed. You can restart by pressing `[s]` or exit the program by pressing `[q]`. ![Program finished example](media/gui_program_finished.png)