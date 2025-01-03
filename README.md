# markov_str

[![crates.io badge]][crates.io] [![docs badge]][docs]

[crates.io badge]: https://img.shields.io/crates/v/markov_str.svg
[crates.io]: https://crates.io/crates/markov_str
[docs badge]: https://docs.rs/markov_str/badge.svg
[docs]: https://docs.rs/markov_str

markov_str is a fast and memory efficient Markov Chain implementation, optimized for text generation.

## Features

- User can specify what regex they want to use for tokenization.
- MarkovChain::generate and MarkovChain::generate_start functions both take RngCore instead of using rand::thread_rng().
- Strings are interned for faster training and less memory usage.
- Serialization and deserialization with [serde](https://docs.rs/serde/latest/serde/), when `serialize` flag is used.

## Example

```rs
let training_path = "data";

// Gets the paths of evey file and directory in the training_path.
let tpaths = fs::read_dir(training_path)
	.unwrap_or_else(|_| panic!("Can't read files from: {}", training_path));

// Only the files remain
let files = tpaths
	.filter_map(|f| f.ok())
	.filter(|f| match f.file_type() {
		Err(_) => false,
		Ok(f) => f.is_file(),
	});

// Reads every file into a string
let contents = files.filter_map(|f| read_to_string(f.path()).ok());

// Creating the Markov Chain
let markov_chain = contents.fold(
	MarkovChain::with_capacity(2, 8_000_000, Regex::new(WORD_REGEX).unwrap()),
	|mut a, s| {
		a.add_text(&s);
		a
	},
);

// Number of tokens
println!("{}", markov_chain.len());

// Generation
for _ in 0..10 {
	println!("{}", markov_chain.generate_start("among the       ", 25).unwrap());
}
```

This example is taken from the `examples/main.rs`, you can run it by:
```sh
./get_data.sh
cargo run --release --example=main
```

`./get_data.sh` will download the first 200 books from [Project Gutenberg](https://www.gutenberg.org/), which totals up to more than 100MBs of text.

## Changelog

[Changelog](CHANGELOG.md)

## License

markov_str is licensed under the MPL-2.0 license. You can use it in both open-source software of different licenses and proprietary software as long as
changes to the original code is shared under the same license.

## Contributing

Feel free to open issues and pull requests. If you want to help with what I am currently working on, take a look at the [Stuff left to do](#stuff-left-to-do) section.

## Stuff left to do

- Multithreading support
- Arena for ChainItems
- Better code documentation
- Even better performance
