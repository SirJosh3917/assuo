# assuo [WIP]

"Assuo" is latin for "patch", meaning to mend, sew, or tack on.
`assuo` is a program, similar to [`patch(1)`](https://man7.org/linux/man-pages/man1/patch.1.html), but it operates differently.
`assuo` operates based on being given a "source". Then, it applies a series of modifications to the source.

# Getting Started

First, install `assuo`. [Install a precompiled release from GitHub](https://github.com/SirJosh3917/assuo/releases), or compile from source by [installing Rust](https://rustup.rs/) and running `cargo install assuo`.

## How it works

`assuo` deals with two things: sources and patches. A source is just some sequence of bytes, and a patch tells assuo how to modify that
sequence of bytes. In addition, `assuo` makes patches easy to sequentially stick on. Patches refer to spots _within the source_ for their
modification, and this position is automatically adjusted so that it gets inserted in the right place even if there are patches applied
before it. A picture is worth a thousand words, so let's dive straight into it.

### Table of Contents

- [Table of Contents](#Table-of-Contents)
- [Hello, World!](#Hello-World)
- [Sources](#Sources)
- [Pre and Post Positioning](#Pre-and-Post-Positioning)

### Hello, World!

```toml
# The source specifies where we get a copy of the source bytes from.
# In our case, we will specify some `text`.
[source]
text = "Hello!"

# Now, we can apply a series of patches. In TOML, the double brackets
# ([[]]) mean an array, so we could add more of these sequentially in
# the file later.
[[patch]]

# We specify what we want to do. Here, we'll say that we want to insert
# some text. We'll make the source go from "Hello!" to "Hello, World!"
do = "insert"

# Way means the direction we insert from. In our small example, this
# doesn't matter, but we'll cover it more later.
way = "post"

# The spot is where we want to insert at. We can find out where we want
# to insert at based on the index in the source bytes.
#
# | H | e | l | l | o | ! |
# ^   ^   ^   ^   ^   ^   ^
# 0   1   2   3   4   5   6
spot = 5

# Now, we have to supply the source for where we get the bytes from.
# You may notice that this looks very similar to the [source] we have at
# the top, and you'd be absolutely right! Any valid [source] is a valid source
# here too.
source = { text = ", World" }
```

### Sources

In our [Hello, World!](#Hello-World) example, we only utilized the `text` source. However, `assuo` supports multiple kinds of sources.

- `bytes`
  Supply an amount of bytes in-line with the file.

```toml
[source]
bytes = [1, 2, 3, 4]
```

- `text`
  Supply a UTF-8 string, which will be converted into bytes and used as the source.

```toml
[source]
text = "Hello!"
```

- `file`
  Supply the path to a file on disk, which will be read and used as the source.

````toml
[source]
file = "./path/to/file"
``

- `url`
  GETs the specified URL, and uses the response body as the source of bytes.

```toml
[source]
url = "https://example.com/"
````

- `assuo-file`
  Reads a file from disk, attempts to interpret it as an assuo config file, compile it, and uses the compiled result as a source of bytes.

```toml
# a.toml
[source]
text = "Hello! From a.toml!"
```

```toml
# b.toml
[source]
assuo-file = "./a.toml"
```

In this case, running `cat b.toml | assuo` should print `Hello! From a.toml!` to the screen.

- `assuo-url`
  GETs the specified URL, attempts to interpret it as an assuo config file, and uses the compiled result as a source of bytes.

```toml
[source]
assuo-url = "https://example.com/"
```

### Pre and Post Positioning

In assuo, patches are applied _sequentially_. For example, if we have two patches that insert into the same position, you will get consistent,
reproducible results every time. In the following example, the output would be `>ba<`.

```toml
[source]
text = "><"

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "a" }

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "b" }
```

It often makes sense to just append patches to the end of an assuo config file, given the nature of assuo. However, if you want to guarantee
that you insert _before_ a given segment, then you must use the `pre` direction. In the example above, there is no way to append a single patch
that guarantees a `c` right before the `<`, without using the `pre` direction. Below is a patch that does just that.

```toml
[[patch]]
do = "insert"
way = "pre"
spot = 1
source = { text = "c" }
```
