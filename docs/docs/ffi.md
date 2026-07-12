---
title: FFI
permalink: /docs/ffi/
---

# FFI

Sometimes people are weak minded individuals who refuse to take advantage of Douglang's superiority over every other language. As such, we allow function calls to C. This is rigged, and it is named that way.

```douglang
Rigged "puts" "Hey Doug"
```

This prints out "Hey Doug". But that's lame, and tts is better.

## Arguments

Arguments follow the function name. They can be:

- String or number literals: `#!douglang Rigged "sleep" 1`
- Doug expressions in parentheses: `#!douglang Rigged "puts" (Doug)`
- Bare Doug notation: `#!douglang Rigged "puts" Doug`

```douglang
Doug set "Hey Doug, why is chat so slow"
Rigged "puts" (Doug)
```
