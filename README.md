# Turing Machine Compiler

This repository contains a compiler which generates Turing Machines from a
simple functional programming language. You can read more about the language in
this [blog post](https://riscadoa.com/compilers/turing-machine-compiler-1/).

## Usage

To compile a `.tmc` program, you must specify the alphabet used by the
generated turing machine. You can do this by passing the `--alphabet` flag to
the compiler.

```bash
$ tmc ./samples/inc.tmc --alphabet '0' '1' '#'
```

This command will then output to `stdout` the generated turing machine in the
chosen format. The [`awmorp`](https://github.com/awmorp/turing) format is used
by the emulator found [here](https://morphett.info/turing/turing.html), so you
can use this emulator to test your program.

## Samples

There are some samples in the `samples` directory which demonstrate some
possible programs.