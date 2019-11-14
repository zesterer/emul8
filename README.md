# Emul8

![alt tag](misc/screenshot.png)

Emul8 is a CHIP-8 emulator written in terse-yet-idiomatic Rust.
It is a reincarnation of an older project of mine, [Emul8or](https://github.com/zesterer/emul8or), a project that I am no longer happy to point to as my sole attempt at emulator development.

Emul8 supports a series of decompilation, introspection and debugging features.

# Running

`emul8 <binary file>`

For example,

`emul8 test/test.ch8`

# Extra Options

```
emul8 0.1.0

USAGE:
    emul8 [FLAGS] [OPTIONS] <input>

FLAGS:
    -d, --debug      Enable debugging features
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bg-color <bg-color>                    The background color to use (in RGB hexadecimal) [default: 00000000]
    -c, --cycles-per-frame <cycles-per-frame>    Specify the number of cycles to execute per frame [default: 250]
    -f, --fg-color <fg-color>                    The foreground color to use (in RGB hexadecimal) [default: FFFFFFFF]
    -f, --flicker-timeout <flicker-timeout>
            The number of frames that a pixel should stay active for to reduce flicker [default: 1]


ARGS:
    <input>    The CHIP-8 binary to execute
```

# Debugging

Emul8 also has primitive debugging tools. Use the `--debug` flat to enable them. These include:

- Displaying the instruction currently executed both in opcode and human-readable form

- Pausing the emulator by pressing `P`

- Displaying register values by pressing `R`

- Displaying values in memory by pressing `M`
