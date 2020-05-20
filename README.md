# oxidized-json-checker

This is a pure Rust version of [the JSON_checker library](http://www.json.org/JSON_checker/).

This is a Pushdown Automaton that very quickly determines if a JSON text is syntactically correct. It could be used to filter inputs to a system, or to verify that the outputs of a system are syntactically correct.

You can use it with [the `std::io::Read` Rust trait](https://doc.rust-lang.org/std/io/trait.Read.html) to checked if a JSON is valid without having to keep it in memory.

## Performances

I ran some tests against `jq` to make sure the library when in the bounds.
I used a big JSON lines files (8.3GB) that I converted to JSON using `jq -cs '.'` 😜

You can find those Wikipedia articles on [the benchmark repository of Paul Masurel's Tantivy](https://github.com/tantivy-search/search-benchmark-game#running).

### `jq type`

How many times does `jq` takes when it comes to checking and determining the type of a JSON document?
Probably too much, and also a little bit of memory: 12GB!

```bash
$ time cat ../wiki-articles.json | jq type
"array"

real    1m55.064s
user    1m37.335s
sys     0m21.935s
```

### `ojc`

How many times does it takes to `ojc`? Just a little bit less! It also consumes 0kb of memory.

```bash
$ time cat ../wiki-articles.json | ojc
Array

real  0m56.780s
user  0m47.487s
sys   0m12.628s
```

### `ojc` with SIMD

How many times does it takes to `ojc` already? 56s, that can't be true, we are in 2020...
What about enabling some SIMD optimizations? Compile the binary with the `nightly` feature and here we go!

```bash
$ cargo build --release --features nightly
$ time cat ../wiki-articles.json | ojc
Array

real    0m15.818s
user    0m10.892s
sys     0m10.721s
```
