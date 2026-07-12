---
title: Compiling to C
permalink: /docs/compiling/
---

# Compiling to C

Running Douglang through the interpreter is fun, but sometimes you want a "real" executable. Our very qualified scientists at **Basement Technologies Inc** learned how to translate Douglang into C.

```
douglang program.doug --compile out.c
```

This produces a standalone `.c` file. If you don't provide an output path, it defaults to `out.c`.

```
gcc out.c -o program
./program
```

## What TTS becomes

As a result of your choice to compile Douglang to an inferior language, you don't get to hear tts. This is entirely your fault. You had a choice.
