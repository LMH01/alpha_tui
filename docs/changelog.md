# Changelog

## v1.1.0

### New feature
- Added new cli option: `--allowed-instructions` (fr #15)
    - This allows to limit the available instructions, more can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md

### Other
- Implemented a limit that limits how many instructions can be run at max (currently 1 million)
    - This ensures that the program will not freeze when an infinite loop is executed
    - Added option `--disable-instruction-limit` with which this limit can be circumvented

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