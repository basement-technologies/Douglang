---
title: Variables
permalink: /docs/variables/
---

# Variables

Here at **Basement Technologies Inc**, we think variables are for the weak. As such, Douglang doesn't have them. In their place, we write the name of our <del>captor</del>benefactor, Douglas Scott Wreden.

```douglang
Doug set "goot"
```

Let's explain that code block. At first glance, it might look like variables. Trust me, it's not. What this is actually doing is indexing a signed array and setting its value to "goot". Yes, you read that right: a signed array. Our scientists were hard at work finding a solution to the modern problem of variables, and this is what they made.

So, what the hell even is a signed array? Well, you know how arrays are indexed like `#!_ my_array[i]`? Well, we thought that it's kinda stupid for indices to only be positive (unsigned). So we poured all of our channel points into developing a new array with negative indices. That means `#!_ array[-10]` is correct. However, Douglang doesn't use that *old*, *smelly*, *decrepit* syntax. Instead, you use ***Doug Notation***! Cue applause.

Doug Notation is really quite simple. You write `#!douglang Doug`. That's it.

So:

```douglang
Doug
```

What index is that? It's obviously 1. Because there's only one Doug.

Doug Notation is similar to Brainfuck's `#!_ <` and `#!_ >`. Don't worry, it's very simple. It only has three rules.

1. **Chained Dougs:** Jump count, in powers of two. `#!douglang Doug` is 1, `#!douglang DougDoug` is 2, `#!douglang DougDougDoug` is 4, `#!douglang DougDougDougDoug` is 8, and so on.

2. **Sequences:** Sequences of Doug Chains combine with each other. They are separated by whitespace.

3. **Even and Odd Chains:** Sequences count the number of chains they have. The sequence `#!douglang DougDoug Doug` has two chains, for instance. And in that sequence, `#!douglang DougDoug` is an odd chain as it is the 1st chain in the sequence, and `#!douglang Doug` is an even chain as it is the second chain.
    - Odd chains: Perform addition.
    - Even chains: Perform subtraction.

    So, `#!douglang DougDoug Doug` performs `#!_ + 2 - 1` on the index. Or, more simply, is equivalent to just writing `#!douglang Doug`.

Simple, right? With that in mind, getting to 11:

```douglang
DougDougDougDougDoug D: 16 :D
DougDougDoug D: -4. Now at 12 :D
Doug D: 1. Now at 13 :D
DougDoug D: -2. Now at 11 :D
```

And if you want to reset the index back to 0: `#!douglang Bald`. Simple, right? That's how you access variables. But how do you use them?
```douglang
set "goot"
```
The set keyword sets the value at the current index. Paired with Doug Notation:
```douglang
Bald set "goot"
```
That sets the 0th index's value to `#!douglang "goot"`. Because `#!douglang set` overrides the previous value, there are other `#!douglang set` keywords that simply change it. They are:

* `#!douglang +set`
* `#!douglang -set`
* `#!douglang *set`
* `#!douglang /set`
* `#!douglang %set`

Note, these do not work like `#!_ +`, `#!_ -`, etc in other languages. They work exactly like `#!douglang set`, except instead of overwriting the value stored at the index, they perform an operation on it. And, that's everything about variables. Or, actually, I'm getting word from the technicians at **Basement Technologies Inc** that there's actually one thing left: Doug Expressions.

Doug Expressions look exactly like regular Doug Notation, with two differences:

1. They're in parenthesis
2. They are always relative to index 0
3. They don't move the index
4. And evaluate to the value stored at that index

<small>And yes, that's definitely two differences. Don't let the numbers fool you.</small>

So, if you were to store something in index 2, you can say either `#!douglang DougDoug tts` to move the index to 2 and tts, but that's unreliable if the index isn't already close to 2, or you want to keep the index pointer where it is. So, instead, you can say `#!douglang tts (DougDoug)` which *always* evaluates to whatever is stored at `#!douglang DougDoug` relative to the current tape cursor.  

---
So, you've been taught enough about variables. You know enough to understand what this does!

```douglang
set "Hey, Doug. "
Doug set 42
Doug set "The answer to life, the universe, and everything is "
Bald +set (DougDoug) +set (Doug) +set "." tts
```

Also, make sure you count properly. Douglang has an irrational fear of people not knowing how to count. This means before you store something at index 4, something has to be at index 3, and so on. Same for negative indices. So, no `#!douglang DougDougDoug` before you have something at `#!douglang DougDoug Doug`.
