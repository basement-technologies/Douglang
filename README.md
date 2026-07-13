# Douglang

**Certified Language of the Basement 2026**

A programming language based on [that one pepper guy](https://youtube.com/@DougDoug) who solves problems no-one has. It's somewhere between being usable and being esoteric, and just like Doug, it hates ***you*** specifically.

> Not actually affiliated with DougDoug.

# Learn

There's [documentation](https://basement-technologies.github.io/Douglang). If you want to write Douglang code, you *have* to read it given the eccentric features. Go on, it won't bite.

# Features

DougLang brings several groundbreaking, innovative, and cutting-edge features to the ~~basement~~table.

##  Doug Notation

Doug Notation replaces variables with "chains" of `Doug` to index an array of values.

Doug Notation serves two purposes: It both evaluates to whatever value is stored at that position, and it also works like Brainfuck's `<` and `>`. To reset the index back to 0, you can use `Bald`. It's weird, but at least `Bald Doug` is a valid expression.

## TTS instead of printing

With the `tts` keyword, you too can feel like Doug. Breath new life upon your debugging, or roleplay being Twitch Chat when Doug isn't live.

```
tts "Hey Doug, I heard that this programming language powers your basement. I'm scared, how have we lasted this long?"
```

If you store a string somewhere, you can also use that.

```
set "Hey Doug, this 5 minute coding adventure has lasted the entire week. Please stop, I'm hungry. I can't go on much longer. I have a wife and children, Doug. I can't just 'clock in five more minutes' at the POGGIES factory. I need a break, Doug. There's nothing poggers about not getting overtime pay."

tts // Providing nothing to TTS uses the value stored in the currently selected index
```

# TTS

Everyone loves TTS. Except for that one guy who doesn't, but we hate him so it evens out.

Currently, TTS uses [`pyttsx3`](https://github.com/nateshmbhat/pyttsx3). It defaults to the Geraint (Male English w/ Welsh accent) voice if it is installed, but it works without that voice. If you don't have the Geraint voice, the company that made these voices (IVONA) was bought out by Amazon. Regardless, you can download it from harposoftware.com, which still sells the voices with a 30 day free trial. Yippee, ephemeral free things!

In the future, I'd like to have support for [Amazon Polly](https://aws.amazon.com/polly/), which is, as far as I am aware, the only other way to use the Geraint voice. I'd appreciate a PR adding support, since I myself don't have an AWS account.

# Contributing

The code is written in Python and is currently a form of spaghetti. It's not al dente, it's actually not even cooked. I'm not sure how the noodles tangled, the scientists at the [Honorable Douglas Academy](https://dougdoug.fandom.com/wiki/Honorable_Douglas_Academy) are still figuring that out. Feel free to submit a PR to improve the structure.
