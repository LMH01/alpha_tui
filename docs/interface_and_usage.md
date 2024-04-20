# Interface and usage

## Commands

The program understands the following commands:

- [load](#load-command)
- [sandbox](#sandbox-command)
- [check](#check-command)

## Load command

The main command to compile and run a program is the `load` command, it takes the file as first parameter. Example: `alpha_tui load examples/programs/faculty.alpha`.

By default the code that is read will be formatted to be easier to read, this can be disabled by using the `--disable-alignment` flag. If you however would like to write the formatted code into the source file you can use the `--write-alignment` flag.

Predetermined breakpoints can be loaded by using the `--breakpoints` flag, it takes multiple line numbers as parameter. Example: `alpha_tui load examples/programs/faculty.alpha -b 5`.

By using the `--custom-instruction-history-file` a file can be provided to the program that contains instructions that should be used to fill the instruction history inside the popup window, where a custom instruction can be entered. When this is supplied, the file is first checked if all instructions that are stored within it are valid. Custom instructions that are run which are not yet contained in this file will be added to it.

To see all arguments that are available with this command use `.\alpha_tui help load`.

When a program is loaded it can look like this: ![Program loaded example](../media/gui_program_loaded.png)

Press `[r]` to begin to run the program, subsequent instructions can also be run with `[r]`. Values that have changed and the line that was run last are highlighted.  This can look like this: ![Program running example](../media/gui_program_running.png)

When the last instruction was executed the following window is displayed. You can restart by pressing `[s]` or exit the program by pressing `[q]`. ![Program finished example](../media/gui_program_finished.png)

### Custom instructions

When in the normal run mode, you can press the `i` key to open up a popup window where a custom instruction can be entered, that should be executed at the current position in the program. You can use the `up` and `down` arrow keys to navigate the history of executed custom instructions. If an instruction is selected in that list, it is executed by pressing `enter`. By typing in the input field you can filter the list. To deselect the list and use the instruction newly written into the text field, press the `up` arrow key, until the list is no longer selected. Pressing `enter` will run the instruction written in the text field.

The popup window can look like this: ![Run custom instruction](../media/gui_program_custom_instruction.png)

Or this if the command history contains elements: ![Run custom instruction with history elements](../media/gui_program_custom_instruction_with_history.png)

If the instruction could not be parsed a simple error is displayed, quit the program with `q` to receive further information on why the instruction could not be parsed.

#### Pitfalls

Using this feature may lead to some unexpected behavior, as the normal program flow is changed. The result might be that the program is broken and runtime errors occur.

Another thing that might occur is, that if a `goto` or `call` instruction is used, the highlighted line might not be the line that was actually executed. This is a visual issue only, it does not effect what instruction is run. After 2-3 steps the highlighted instruction should match the executed instruction again.

### Debug features

Some debug features require you to select a line in which a debug action should take place.

You can enter debug select mode by pressing `[d]`, this could look like this: ![Debug select mode](../media/gui_debug_select_mode.png)

Navigate by using the `arrow keys`, for ease of use `[w]` and `[s]` are also supported.

#### Breakpoints

Breakpoints can be set to run all lines of code up until the line in which the breakpoint is set.

To set a breakpoint enter `debug select mode` and press `[t]` in the line in which you want to set the breakpoint. A star to the left will indicate where a breakpoint is placed.

A placed breakpoint can look like this: ![Breakpoint set](../media/gui_breakpoint_set.png)

Press `[n]` when in run mode to make the program run automatically to the next breakpoint (note how the values to the right have been updated): ![Next breakpoint](../media/gui_breakpoint_mode_run.png)

#### Jump to line

When in `debug select mode` you can select a line and jump directly to it using `[j]`, skipping all other instructions. You should however be careful when using this, because runtime errors are far more likely to occur due to uninitialized accumulators or memory cells.
Functions may also no longer be properly exited because of a misaligned call stack.

## Sandbox command

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

### Error handling

[Miette](https://github.com/zkat/miette) is used for error handling, this provides helpful error messages when a program can not be compiled due to an unknown instruction.

Such error could look like this ![Miette error handling](media/miette_error.png)

or this: 

![Miette error handling](media/miette_error_2.png)