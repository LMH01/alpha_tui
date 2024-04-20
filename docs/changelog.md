# Changelog

## v1.3.0

### New feature

- Next instruction is now displayed in the tui (fr #34)
- The call stack can now be displayed in the tui. Its default state is determined if a call instruction is used. If it is used, it is shown, if such instruction is absent, it is hidden. With the `c` key this display can be hidden or shown manually (fr #34).
- Run custom instruction: it is now possible to use the TAB key to fill in the selected value into the text field

### TUI

- Made TUI behave more dynamic, these changes include:
    - Text inside popup messages now wraps into a new line, if not enough space is available
    - Most block title change the displayed text based on available space
    - Accumulators/Memory Cells column is now always at least 10 characters wide
    - Locked width of breakpoint block to be always 5 characters wide
- Blocks of memory cells, accumulator and stack are now colored light blue
- Changed width of custom instruction popup from 60% screen size to 43%

### Other

- Reworked how predetermined memory values are handled (fr #29):
    - Renamed cli option `--memory-cells-file` to `--memory-config-file`
    - This memory config file can now be used to set the values of `--accumulators`, `--gamma_accumulator` in addition to the values of `--memory_cells` and `--index_memory_cells`.
    - `--memory-config-file` now conflicts with `--accumulators`, `--enable-gamma-accumulator`, `--memory-cells` and `--index-memory-cells`
    - Changed file data type to `json`
- `--disable-memory-detection` no longer forces the usage of either `--accumulators` and `--memory-cells` or `--memory-config-file` (previously `--memory-cells-file`) (fr #30 and #31)
- It is now allowed to write `=` instead of `:=` when writing assignment instructions. Note, however, that this is a deviation from the alpha notation standard (fr #27).
- Removed unnecessary 30ms sleep after key input
- Updated dependencies

### Bug fixes

- Fixed panic when jump to line was used in the first line (#37)
- Fixed panic when very little space is available in the terminal to display tui
- Fixed wrong keybinding hints being displayed when runtime error occurred
- Fixed cli argument `--memory-cells` allowing values that contain numbers only, as these values are conflicting with index memory cells
- [Windows] fixed double key input (#35)

## v1.2.0 (latest version)

### New feature
- Added way to run a single custom instruction while program is running (fr #24 and #25) (error information here is however not as useful as the errors displayed while the program is being build initially)
- Added new command line argument `--custom-instruction-history-file` for subcommand `load`
    - This command allows loading of instructions from a file that are used to fill the instruction history when running a custom instruction
    - If this is set, instructions not yet included in this file are written to it, when entered in the tui

### Other
- Updated dependencies
- Reworked how keybinding hints are displayed, they now automatically wrap into a new line, if the space in one line is not enough to fit all active keybinding hints
- Removed distinction between runtime errors where jump line was used and where it was not
- Updated message when runtime error occurs to give hint that further information on the error is available when program is closed

### Bug fixes
- Fixed URL in errors `InvalidExpression`, `UnknownInstruction` and `MissingExpression` to point to the correct page

## v1.1.0

### New feature
- Added new cli option: `--allowed-instructions` (fr #15)
    - This allows to limit the available instructions, more can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md
- Added new cli option: `--allowed-comparisons` (fr #17)
    - This allows to limit the available comparisons, more can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md
- Added new cli option: `--allowed-operations` (fr #16)
    - This allows to limit the available operations, more can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md

### Other
- Implemented a limit that limits how many instructions can be run at max (currently 1 million)
    - This ensures that the program will not freeze when an infinite loop is executed
    - Added option `--disable-instruction-limit` with which this limit can be circumvented
- Keybind hint for `n` is now dynamically set to `Run to end [n]` when no breakpoint is set or to `Next breakpoint [n]` when at least one breakpoint is set

## v1.0.1

### Other
- Made runtime build errors consistent with runtime errors

### Bug fixes
- Memory cell names could contain non alphabetic or numeric characters

## v1.0.0

### New feature

- Added prettier formatting for code in code window (can be disabled with `-d`)
    - Pretty formatting can be written into the source file
- Added support for index memory cells
- Added support for special accumulator gamma
- Added debug feature: continue execution at line
    - For that breakpoint mode has been renamed to debug select mode and keybindings have been adjusted
- Added new subcommand: `check \<File\> compile`
    - Used to check if program can be compiled

### Other
- Cli arguments have been reorganized and subcommands have been added
    - To load a program the new `load` subcommand can be used
- Updated some dependencies

## v1.0.0-pre-release-1

- initial release