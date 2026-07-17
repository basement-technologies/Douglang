---
title: Compiling to C
permalink: /docs/compiling/
---

# Compiling to C

Running Douglang through the interpreter is fun, but sometimes you want a "real" executable. Our very qualified scientists at **Basement Technologies Inc** learned how to translate Douglang into C.

```
douglang program.doug --compile
```

This produces a standalone `.c` file with the same name as your input file.

```
gcc out.c -o program
./program
```

## Compiling to a Binary

You can skip the manual gcc step and compile straight to a binary with `--cc`:

```
douglang program.doug --cc
```

This writes a C file and compiles it with gcc in one step.

## Libraries

You can link external libraries at both compile time and runtime using the Rigged FFI.

### Compile-time linking

The `--link` flag passes libraries to gcc:

```
douglang program.doug --cc program --link m
```

This links against libm (math), so you can call functions like `Rigged "sin" 0.5`.

```
douglang program.doug --cc program --link m pthread
```

You can also link `.so`, `.dll`, `.dylib`, or `.a` files by passing the full path

```
douglang program.doug --cc program --link ./whyischatsoslow.so
douglang program.doug --cc program --link ./baldat2dplatformers.dll
```

### Runtime linking (interpreter)

The same `--link` flag works when running through the interpreter:

```
douglang program.doug --link m
```

### pkg-config

If a library has a pkg-config file, it will be used automatically.

```
douglang program.doug --cc program --link sdl2
```

This runs `pkg-config --cflags sdl2` and `pkg-config --libs sdl2` under the hood.
