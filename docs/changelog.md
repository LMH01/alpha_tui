# Changelog

## v1.2.0

### New feature
- Added way to run a single custom instruction while program is running (fr #24 and #25) (error information here is however not as useful as the errors displayed while the program is being build initially)

### Other
- Updated dependencies
- Reworked how keybinding hints are displayed, they now automatically wrap into a new line, if the space in one line is not enough to fit all active keybinding hints

### Bug fixes
- Fixed URL in errors `InvalidExpression`, `UnknownInstruction` and `MissingExpression` to point to the correct page

## v1.1.0 (latest version)

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