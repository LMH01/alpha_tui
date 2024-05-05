# CLI Options

For a full list of options and more explanation see `alpha_tui help`, `alpha_tui help load`, `alpha_tui help playground` or `alpha_tui help check`.

For an explanation of the available commands see [interface and usage](interface_and_usage.md).

## General options

Numbered accumulators, the gamma accumulator, memory cells and index memory cells are automatically created when the input program is read.
To circumvent that you can set the option `--disable-memory-detection`. You then need to specify the accumulators, memory_cells and index_memory_cells that should be created. The options `-a`, `-m` and `-i`, or `--memory-config-file` can be used to specify those values. The gamma accumulator has to be enabled using `-g true`.
Note that it is not required to set these values but if a memory type is used that does not exist, the runtime will fail to build, or the custom instruction will cause an error.

If you require accumulators, the gamma accumulator, memory cells or index memory cells to be pre initialized you can use the option `--memory-config-file` to read in a file that contains information about this data. An example for such file can be found [here](../examples/memory_config.json). See [below](cli.md#memory-config-file) for more information on this option.

### Allowed instructions

You can use the option `--allowed-instructions-file` to specify a file where allowed instructions are stored. When this option is provided, all programs will fail to build that contain instructions that are not included in the file.

This makes it possible to challenge yourself into working with only a limited instruction set.

#### How it works

All instructions that are understood by the program can be used to specify what instructions should be allowed, in addition to that there are some shortcuts available:

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

**It is important to understand that only the type of instruction is limited by this option, to specifically limit what memory locations or what comparisons/operations are available you can use the options `-a`, `-g`, `-m` and `-i` or `--memory-config-file`, and `--allowed-comparisons` and `--allowed-operations` . This means that even though you might write `p(h1)` in the allowed instructions file, all available memory cells are allowed in this position, not just `p(h1)`!**

An example file can be found here: [examples/allowed_instructions.txt](../examples/allowed_instructions.txt);

If a runtime can not be built, because certain instructions are not allowed, this could be the result:

![Runtime build error because certain instructions are not allowed](../media/miette_error_instruction_not_allowed.png)

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

**Note**: At least one comparison needs to be specified, if you would like to prevent the use of any comparison you can use `--allowed-instructions-file` to limit the available instructions into only allowing instructions which don't take any comparisons.

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

**Note**: At least one operation needs to be specified, if you would like to prevent the use of any operation you can use `--allowed-instructions-file` to limit the available instructions into only allowing instructions which don't take any operations.

## Memory config file

The option `--memory-config-file` can be used to specify the path to a `JSON` formatted file that contains information about accumulators, the gamma accumulator, memory cells and index memory cells. It can be used to specify values that should be available when the program is started, it can be used to specify what memory types should be available and is can be used to set what memory types should be auto-detectable, meaning that memory values are automatically created if they are missing. Disabling of automatic detection by `--disable-memory-detection` is overwritten when the `autodetection` field is set to true. To enable a specific memory type, create a new entry in the corresponding map. If the value is `null` the memory type is created but no value is set (does not apply to the gamma accumulator). The gamma accumulator can be enabled by setting the `enabled` field to `true`. Its value can be set by using the `value` field, set it to `null` to enable the gamma accumulator but to not assign it any value.
An example file could look like this:

```json
{
    "accumulators": {
        "values": {
            "0": 10,
            "1": null,
            "2": null
        },
        "autodetection": true
    },
    "gamma_accumulator": {
        "enabled": true,
        "value": 10,
        "autodetection": true
    },
    "memory_cells": {
        "values": {
            "h2": null,
            "h1": 10
        },
        "autodetection": true
    },
    "index_memory_cells": {
        "values": {
            "1": null,
            "0": 10
        },
        "autodetection": false
    }
}
```

This file can also be found [here](../examples/memory_config.json).

## Instruction history

The option `--custom-instruction-history-file` can be used to specify a file that should be used to save the command history that is entered in `run custom instruction` popup and the playground mode. If instructions are already contained in that file, it is checked if they are valid, before the tui is opened. The contained instructions are then displayed in the `History` section and can be selected using the up and down arrow keys. 

If a new instruction is written in the tui that is valid, it is added to the file.

(If the file does not exist, a new file is created.) - needs to be implemented

## Examples

### Maximum limitation

If you would like to limit what instructions, comparisons/operations and memory locations are available you can do something like this:

```
alpha_tui load program.alpha
    --disable-memory-detection         // Disable automatic memory detection
    --accumulators 2                   // Enable two accumulators (a0 and a1)
    --enable-gamma-accumulator true    // Enable the gamma accumulator
    --memory-cells "h1,h2"             // Enable memory cells h1 and h2
    --index-memory-cells "1,2"         // Enable index memory cells 1 and 2
    --allowed-comparisons "eq,neq"     // Allow equal and not equal comparisons
    --allowed-operations "add,sub"     // Allow addition and subtraction operations
    --allowed-instructions-file <path> // Allow some specific instructions
```
(The different options have been written each in a new line for better understanding, in the correct command you would write them in one line.)

If run, all programs that don't fulfill these requirements will fail to build.
