+++
title = "Nix Rust Script"
description = """
  How to get the scripting experience while using Rust
"""
date = 2025-02-03
draft = false

[taxonomies]
tags = ["Nix", "Programming"]
[extra]
toc = true
+++

## One Dependency Scripts

I recently [read](https://nixos.wiki/wiki/Nix-shell_shebang) that `nix` can be used in script shebangs.
Furthermore, `nix` can delegate the interpretation of the script to something else.
This allows us to get the same benefit of development environments, but for single files rather than entire projects.

For example, let's say you want to make a `Python` script but don't want to install `Python` globally.
Maybe you don't use it in whatever environment you're using the script in either, so it wouldn't make sense to tack it on as an environment dependency.
You can just write your script and put a shebang at the top:

```python
#! /usr/bin/env nix-shell
#! nix-shell -i python3 -p python3

print("Sick script!")
```

Assuming that you have `nix` installed, you can just execute this script like you would any other.
Neat!

However, this has a drawback.
Because the `nix` expression is evaluated every single time you run the script, startup can be quite slow (around half a second).
This might not sound like much, but this half second makes it feel as slow as molasses.

Enter `flakes`.
Rather than using the old `nix-shell`, we can use the new flake based `nix shell`.

```python
#! /usr/bin/env nix
#! nix shell nixpkgs#python3 --command python

print("Sick script!")
```

With that small change, after the first time we execute the script, it gets cached.
Subsequent runs will use that cache, making startup nearly instantaneous.

With that, we can run any interpreted language easily, quickly, and without installing the dependencies anywhere outside of the script.
But what if we wanted to run *compiled languages*?
Specifically, what if we wanted to use `Rust` for scripting?

## Why Rust?

At first glance, `Rust` may seem like a strange choice for scripting.
After all, with short running scripts, memory safety aren't nearly as important as they would be in longer running applications.
In addition, `Rust` can be quite verbose, particularly with the default error handling.
Plus, just to get "Hello world" we need two nested directories and a `Cargo.toml`.

However, I find the benefits to outweigh the cons.

While memory safety isn't as important in short running programs, having high confidence that your script will do what you want before actually running it is very nice.

For error handling we can use `anyhow`, which was almost made for this exact purpose!

To get around needing all the project boilerplate to run a "Hello world", we can use [rust-script](https://rust-script.org/).
This handles all the boilerplate for us and allows specifying dependencies from within the script itself.

Speaking of dependencies, there's a *big* reason to use `Rust` -- we can avoid reinventing the wheel or pulling in command-line tools with disparate interfaces.
If everything stays in `Rust`, it becomes a lot easier to reason about what the different parts of our script are doing.

And, of course, `Rust` is super quick at runtime.
We'll never encounter a situation where we write a script, only to realize that the performance is just too bad in whatever language we're using, so we have to rewrite it in a more performant language.
`Rust` **is that language** that I would rewrite said script in.

## Usage

```rust
#!/usr/bin/env nix
/*
#!nix shell nixpkgs#clang nixpkgs#cargo nixpkgs#rustc nixpkgs#rust-script --command rust-script
*/

println!("Sick script!");
```

Yes, the multi-line comment is required.
No, I'm not really sure how that works.

We can specify dependencies in one of two ways.
The short way:

```rust
#!/usr/bin/env nix
// cargo-deps: anyhow="1.0.95"
/*
#!nix shell nixpkgs#clang nixpkgs#cargo nixpkgs#rustc nixpkgs#rust-script --command rust-script
*/

use anyhow::{Result, Context};

fn main() -> Result<()> {
  Ok(())
}
```

Or the long way:

```rust
#!/usr/bin/env nix
// cargo-deps: anyhow="1.0.95"
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.95"
//! ```
/*
#!nix shell nixpkgs#clang nixpkgs#cargo nixpkgs#rustc nixpkgs#rust-script --command rust-script
*/

use anyhow::{Result, Context};

fn main() -> Result<()> {
  Ok(())
}
```

I prefer the long way because I don't mind vertical space being used up.
You can also put other things in there, like feature flags.

Running a script for the *very* first time can take a while depending on what dependencies you need to download.
Every time after is super quick, even when recompilation is needed.

Something neat to note is that compiler warnings aren't printed on subsequent runs if the script hasn't been changed since it's not being recompiled!

On my machine, running the following `Rust` script 250 times in a row takes 21 seconds:

```rust
#!/usr/bin/env nix
// cargo-deps: anyhow="1.0.95"
/*
#!nix shell nixpkgs#clang nixpkgs#cargo nixpkgs#rustc nixpkgs#rust-script --command rust-script
*/

use anyhow::{Result, Context};

fn main() -> Result<()> {
  Ok(())
}
```

Running the following `Python` script 250 times in a row takes my machine 17 seconds:

```py
#! /usr/bin/env nix
#! nix shell nixpkgs#python3 --command python

pass
```

And, with `python3` installed, running a `Python` script containing just `pass` 250 times takes 3 seconds.

For less than a tenth of a second difference, self-contained scripts with `nix flakes` are very worth it for interactive use.
