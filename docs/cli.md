# CLI Options

For a full list of options and more explanation see `alpha_tui help`, `alpha_tui help load`, `alpha_tui help sandbox` or `alpha_tui help check`.

For an explanation of the available commands see [interface and usage](interface_and_usage.md).

## General options

Accumulators and memory cells are automatically created when the input program is read.
To circumvent that you can set the option `--disable-memory-detection`. You then need to specify the accumulators and memory_cells that should be created. The options `-a` and `-m` or `--memory-config-file` can be used to specify those values. Note that it is not required to set these values but if a memory type is used that does not exist, the runtime will fail to build.

If you require accumulators, the gamma accumulator or memory cells to be pre initialized you can use the option `--memory-config-file` to read in a file that contains information about this data. This file is formatted in json. To enable a specific memory type, create a new entry in the corresponding map. If the value is `null` the memory type is created but no value is set (does not apply to the gamma accumulator). The gamma accumulator can be enabled by setting the `enabled` field to `true`. Its value can be set by using the `value` field, set it to `null` to enable the gamma accumulator but to not assign it any value. An example for such file can be found [here](../examples/memory_config.json).

### Allowed instructions

You can use the option `--allowed-instructions` to specify a file where allowed instructions are stored. When this option is provided, all programs will fail to build that contain instructions that are not included in the file.

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

**It is important to understand that only the type of instruction is limited by this option, to specifically limit what memory locations or what comparisons/operations are available you can use the options `-a`, `-m` or `--memory-config-file` and `--allowed-comparisons` and `--allowed-operations` . This means that even though you might write `p(h1)` in the allowed instructions file, all available memory cells are allowed in this position, not just `p(h1)`!**

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