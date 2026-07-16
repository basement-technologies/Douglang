---
title: Control Flow
permalink: /docs/control_flow/
---

# Control Flow

Douglang doesn't have much control flow. Our scientists are still figuring out what "structured programming" even means. That means no functions or fancy loops. I'm sure we'll get them eventually, it's only a matter of time.

But let's focus on the present, because the present is a gift, that's why it's called the present.

To loop, say `#!douglang loop`. Confusing, I know. Don't worry, I believe in you.
```douglang
loop [
    tts "I'm going to say this forever."
]
```
Loops go forever. Why do they go forever? Because if they didn't loop, they wouldn't be a `#!douglang loop`. You can end a loop with `#!douglang guoD`:
```douglang
loop [
    tts "What is my purpose? Just to loop? Forever?"
    guoD D: Oh, thank you. :D
]
```
Except now it doesn't loop and therefore doesn't deserve to be called `#!douglang loop`. We can fix this with predictions. 
```douglang
prediction (Doug) = "Douglas Wreden" [
    Believers win [
        tts "Hey Doug"
    ]
]
```


And, what's that? `#!_ =` is for assignment? No, This is Douglang. We use `#!douglang set` for assignment. That frees up `#!_ =` for conditionals, instead of the *disgusting* and *vulgar* `#!_ ==`.

The `#!douglang Believers win` block runs whenever the condition is true, because the Believers believe it will evaluate to true, and they won. The same would happen with a `#!douglang Doubters win` block, because the Doubters believe it will evaluate to false.

Combine that with `#!douglang loop` and now you have a loop that doesn't go on forever. I don't know why you would want that, because loops going forever is cool.

```douglang
D: Laundry alarm :D
Doug set 5
loop [
    Bald set "Laundry in " +set (Doug) +set "." tts
    Doug -set 1
    prediction (Doug) = 0 [
        Believers win [
            guoD
        ]
    ]
]
tts "LAUNDRY!!!!!!!" 
```
