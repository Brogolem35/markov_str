//! Fast and memory efficient Markov Chain implementation, optimized for text generation
//!
//! Features
//! --------
//!
//! - User can specify what regex they want to use for tokenization.
//! - MarkovChain::generate and MarkovChain::generate_start functions both take RngCore instead of using rand::thread_rng().
//! - Strings are interned for faster training and less memory usage.
//! - Serialization and deserialization with [serde](https://docs.rs/serde/latest/serde/), when `serialize` feature flag is used.
//!
//! Example
//! -------
//!
//! ```rust
//! use markov_str::*;
//! use rand::SeedableRng;
//! use regex::Regex;
//! use std::fs::{self, read_to_string};
//!
//! let training_path = "data";
//!
//! // Gets the paths of evey file and directory in the training_path.
//! let tpaths = fs::read_dir(training_path)
//!     .unwrap_or_else(|_| panic!("Can't read files from: {}", training_path));
//!
//! // Only the files remain
//! let files = tpaths
//!     .filter_map(|f| f.ok())
//!     .filter(|f| match f.file_type() {
//!         Err(_) => false,
//!         Ok(f) => f.is_file(),
//!     });
//!
//! // Reads every file into a string
//! let contents = files.filter_map(|f| read_to_string(f.path()).ok());
//!
//! // Creating the Markov Chain
//! let markov_chain = contents.fold(
//!     MarkovChain::with_capacity(2, 8_000_000, Regex::new(WORD_REGEX).unwrap()),
//!     |mut a, s| {
//!         a.add_text(&s);
//!         a
//!     },
//! );
//!
//! // Number of tokens
//! println!("{}", markov_chain.len());
//!
//! // Generation
//! for _ in 0..10 {
//!     println!("{}", markov_chain.generate_start("among the       ", 25, &mut rand::thread_rng()).unwrap());
//! }
//! // Generation
//! println!("{}", markov_chain.len());
//!
//! // ThreadRng
//! for _ in 0..10 {
//!     println!(
//!         "ThreadRng: {}",
//!         markov_chain
//!             .generate_start("among the       ", 25, &mut rand::thread_rng())
//!             .unwrap()
//!     );
//! }
//!
//! // StdRng with seed
//! let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
//! for _ in 0..10 {
//!     println!(
//!         "Seeded: {}",
//!         markov_chain
//!             .generate_start("among the       ", 25, &mut rng)
//!             .unwrap()
//!     );
//! }
//!
//! // Cloned
//! let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
//! let m: MarkovChain = markov_chain.clone();
//! for _ in 0..10 {
//!     println!(
//!         "Cloned: {}",
//!         m.generate_start("among the       ", 25, &mut rng).unwrap()
//!     );
//! }
//! ```
//!
//! This example is taken from the `examples/main.rs`, you can run it by:
//! ```ignore
//! ./get_data.sh
//! cargo run --release --example=main
//! ```
//!
//! `./get_data.sh` will download the first 200 books from [Project Gutenberg](https://www.gutenberg.org/), which totals up to more than 100MBs of text.
//!
//! License
//! -------
//!
//! markov_str is licensed under the MIT license. Feel free to fork and use however you like.

mod chain;
pub use crate::chain::*;

/// Recommended Regex for general use.
pub static WORD_REGEX: &str = r"(\p{Alphabetic}|\d)(\p{Alphabetic}|\d|'|-)*(\.|!|\?)?";