# assuo

"Assuo" is latin for "patch", meaning to mend, sew, or tack on.
`assuo` is a program, similar to [`patch(1)`](https://man7.org/linux/man-pages/man1/patch.1.html), but it operates differently.
`assuo` operates based on being given a "source". Then, it applies a series of modifications to the source.

# Getting Started

First, install `assuo`. [Install a precompiled release from GitHub](https://github.com/SirJosh3917/assuo/releases), or compile from source by [installing Rust](https://rustup.rs/) and running `cargo install assuo`.

## How it works

`assuo` will, given a source, apply a series of modifications to the source. An example of an assuo file could look as follows:

```toml
[source]
file = "hello.txt"

[[patch]]
type = "insert after"
spot = 40
source = { url = "https://www.google.com/" }
```
