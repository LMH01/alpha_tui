# Changelog

## v1.5.0 (latest version)

### New feature

- Added syntax highlighting for instructions, this can be disabled using the new `--disable-syntax-highlighting` flag (fr #8)
- The theming of the app can now be customized (fr #77)
    - For now there are 3 themes build in, use `-t` to set the theme that should be loaded:
        - dracula (default)
        - default-old (this was the color scheme before this update)
        - gray
    - Dracula is now the default color scheme
    - Custom themes can be written as a `JSON` file that can be loaded with `--theme-file`
    - A `theme.json` file can be placed in `$HOME/.config/alpha_tui` to load it automatically when the program starts

### TUI

- Changed color of list highlight to be in line with dracula color scheme
- Currently run instruction is no longer highlighted in bold
- Full-line comments starting with `#` are now hidden (fr #73)
- Lines only containing comments starting with `//` will no longer be formatted to be in line with other comments, they now always start at the beginning of the line
- Args `--accumulators`, `--memory-cells` and `--index-memory-cells` are no longer valid for playground mode (fr #70)

### Other

- `a`. `p` and `y` are now always converted to their greek alphabet equivalents in a loaded program. Using the `--write-alignment` flag these symbols can be written to the source file
- All `a` and $\alpha$ are now displayed as $\alpha0$
- Resetting the runtime will now restore the memory values that where set when the tui was launched, this means that memory values configured using a memory config file are now restored when the runtime is reset (#74)

### Bug fixes

- It was possible to create memory cells with a label that consisted only of numbers when the memory cell was defined in the memory config file
- Fixed `r` keybinding hint not updating to `Run to next breakpoint` if breakpoint was set

## v1.4.1

### Bug fixes

- A `build_allowed_instructions_error` would occur, when the `load` command was used while an `--allowed-instructions-file` was provided

## v1.4.0

### New feature

- Added `run` subcommand to `check` subcommand, with that subcommand it can be checked if program can be run without an error

### TUI

- Added α symbols in front of each entry for a "numerical accumulator" in the accumulator field of the tui (fr #47) (by [@reeelix](https://github.com/reeelix))
- Reduced size of `Execution finished` popup (fr #46)
- The escape key can now be used to exit the program in every situation, except when a custom instruction is entered in the load mode (fr #51)

### Command line arguments

- Renamed cli option `--allowed-instructions` to `--allowed-instructions-file` (fr #49) (by [@reeelix](https://github.com/reeelix))
- Improved order of command line options in `.\alpha_tui load --help` (fr #54) (by [@reeelix](https://github.com/reeelix))
- Added short forms `-i` for `--index-memory-cells` and `-g` for `--enable-gamma-accumulator` (fr #56) (by [@reeelix](https://github.com/reeelix))
- Option `--enable-gamma-accumulator` no longer takes a boolean as value, the value is now set by just using this option
- Help messages when a runtime error occurs because a memory type is missing now also hint towards the memory config file
- Formatting for the file provided to `--memory-config-file` has been updated to include values of automatic detection if the values should be enabled (fr #62)
- Allowed instructions file can now be used to specify allowed comparisons and operations in addition to allowing instructions. For that the file formatting was change to `.json`. This makes it now possible to forbid all comparisons/operations (fr #72)

### Other

- Error messages now hint towards missing blank spaces (fr #58) (by [@reeelix](https://github.com/reeelix))
- If the gamma accumulator is completely disabled (meaning autodetection is disabled and it does not exist from the start) `p(y)` is treated as access of memory cell with label `y` instead of taking the value of the gamma accumulator and using it to access an index memory cell (fr #64)

### Bug fixes

-  `%` was missing from help message when `runtime_error::unknown_operation` occurred
- `Press [ENTER] to close` missing from `unable to parse instruction` error, when screen width was below certain threshold
- Fixed accumulator 0 not changing value to calculated value when stack op is calculated due to side effect in stack op operation (fr #44)
- NOOP was treated as instruction that could be forbidden, so programs with empty lines could fail to build if `--allowed-instructions` (or now `--allowed-instructions-file`) was set (fr #52)
- `check` subcommand would mark a test as successful even though labels where missing
- Run custom instruction could be used in load mode to execute instructions, operations and comparisons that where forbidden (fr #63)

## v1.3.0

### New feature

- Added playground mode (run with `.\alpha_tui playground`)(fr #33)
- Next instruction is now displayed in the tui (fr #34)
- The call stack can now be displayed in the tui. Its default state is determined if a call instruction is used. If it is used, it is shown, if such instruction is absent, it is hidden. With the `c` key this display can be hidden or shown manually (fr #34).
- Run custom instruction: it is now possible to use the TAB key to fill in the selected value into the text field

### TUI

- Made TUI behave more dynamic, these changes include:
    - Text inside popup messages now wraps into a new line, if not enough space is available
    - Most block titles change the displayed text based on available space
    - Accumulators/Memory Cells column is now always at least 10 characters wide
    - Locked width of breakpoint block to always be 5 characters wide
- Blocks of memory cells, accumulators and stack are now colored light blue
- Changed width of custom instruction popup from 60% screen size to 43%

### Command line arguments

- Reworked how predetermined memory values are handled (fr #29):
    - Renamed cli option `--memory-cells-file` to `--memory-config-file`
    - This memory config file can now be used to set the values of `--accumulators`, `--gamma_accumulator` in addition to the values of `--memory_cells` and `--index_memory_cells`.
    - `--memory-config-file` now conflicts with `--accumulators`, `--enable-gamma-accumulator`, `--memory-cells` and `--index-memory-cells`
    - Changed file data type to `json`
- `--disable-memory-detection` no longer forces the usage of either `--accumulators` and `--memory-cells` or `--memory-config-file` (previously `--memory-cells-file`) (fr #30 and #31)
- command line arguments `--disable-memory-detection`, `--allowed-comparisons`, `--allowed-operations`, `--enable-gamma-accumulator` and `--allowed-instructions-file` are no longer available for all commands because the new `playground` command does not make use of them. They are thus no longer displayed in `.\alpha_tui help`, instead they are now explained in either `.\alpha_tui help load` or `.\alpha_tui help check`.
- the file pointed to by `--custom-instruction-history-file` is now created, if it did not exist

### Other

- It is now allowed to write `=` instead of `:=` when writing assignment instructions. Note, however, that this is a deviation from the alpha notation standard (fr #27).
- If a value is assigned to an accumulator or a memory cell that does not exist yet, using the custom instruction feature, it is now automatically created (except when `--disable-memory-detection` is set) (fr #43)
- The labels `End` and `Ende` can be used to end the program
- Removed unnecessary 30ms sleep after key input
- Updated dependencies

### Bug fixes

- Fixed panic when jump to line was used in the first line (#37)
- Fixed panic when very little space is available in the terminal to display tui
- Fixed panic when trying to assign a calculated value to an accumulator or a memory cell that does not exist
- Fixed wrong keybinding hints being displayed when runtime error occurred
- Fixed cli argument `--memory-cells` allowing values that contain numbers only, as these values are conflicting with index memory cells
- Fixed wrong line highlighted when custom call instruction was run
- Fixed dismiss run completed popup message keybinding hint not vanishing when message was dismissed
- Fixed rare panic that could happen on specific conditions when an error occurred while an instruction was parsed
- [Windows] fixed double key input (#35)

## v1.2.0

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
- Added new cli option: `--allowed-instructions-file` (fr #15)
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
