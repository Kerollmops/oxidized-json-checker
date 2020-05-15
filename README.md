# oxidized-json-checker

This is a pure Rust version of [the JSON_checker library](http://www.json.org/JSON_checker/).

This is a Pushdown Automaton that very quickly determines if a JSON text is syntactically correct. It could be used to filter inputs to a system, or to verify that the outputs of a system are syntactically correct.

You can use it with [the `std::io::Read` Rust trait](https://doc.rust-lang.org/std/io/trait.Read.html) to checked if a JSON is valid without having to keep it in memory.
