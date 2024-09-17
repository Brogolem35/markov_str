//! Fast and memory efficient Markov Chain implementation, optimized for text generation
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
//! This example is taken from the `src/main.rs`, you can run it by:
//! ```ignore
//! ./get_data.sh
//! cargo run --release
//! ```
//!
//! `./get_data.sh` will download the first 200 books from [Project Gutenberg](https://www.gutenberg.org/), which totals up to more than 100MBs of text.
//!
//! License
//! -------
//!
//! markov_str is licensed under the MIT license. Feel free to fork and use however you like.

use hashbrown::HashMap;
use lasso::{Capacity, Rodeo, Spur};
use rand::{seq::SliceRandom, RngCore};
use regex::Regex;

#[cfg(feature = "serialize")]
use {
	serde::{Deserialize, Serialize},
	serde_json_any_key::*,
};

/// Represents a Markov Chain that is designed to generate text.
///
/// [Wikipedia](https://en.wikipedia.org/wiki/Markov_chain)
#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct MarkovChain {
	#[cfg_attr(feature = "serialize", serde(with = "any_key_map"))]
	items: HashMap<Vec<Spur>, ChainItem>,
	state_size: usize,
	#[cfg_attr(feature = "serialize", serde(with = "serde_regex"))]
	regex: Regex,
	cache: Rodeo,
}

impl MarkovChain {
	/// Creates an empty MarkovChain.
	///
	/// The hashmap and the cache of the MarkovChain is initially created with the capacity of 0.
	/// It will not allocate until the first insertion.
	pub fn new(state_size: usize, regex: Regex) -> MarkovChain {
		MarkovChain {
			items: HashMap::new(),
			state_size,
			regex,
			cache: Rodeo::new(),
		}
	}

	/// Creates an empty MarkovChain with the specified capacity.
	///
	/// The hashmap and the cache of the MarkovChain will be able to hold at least `capacity` elements without
	/// reallocating. If `capacity` is 0, the hash map will not allocate.
	pub fn with_capacity(state_size: usize, capacity: usize, regex: Regex) -> MarkovChain {
		MarkovChain {
			items: HashMap::with_capacity(capacity),
			state_size,
			regex,
			cache: Rodeo::with_capacity(Capacity::for_strings(capacity)),
		}
	}

	/// Adds text as training data. The tokens will be created with the regex of the MarkovChain.
	pub fn add_text(&mut self, text: &str) {
		let tokens: Vec<_> = self
			.regex
			.find_iter(text)
			.map(|t| self.cache.get_or_intern(t.as_str()))
			.collect();

		// Creating a preallocated buffer and filling and cleaning it instead of creating a new one every loop is way more efficient.
		let mut prevbuf: Vec<Spur> = Vec::with_capacity(self.state_size);
		for win in tokens.windows(tokens.len().min(self.state_size + 1)) {
			let rel = win.last().unwrap();

			for i in 1..win.len() {
				prevbuf.clear();
				for t in win.iter().rev().skip(1).take(i).rev() {
					prevbuf.push(*t);
				}

				if let Some(ci) = self.items.get_mut(&prevbuf) {
					ci.add(*rel);
				} else {
					self.items.insert(prevbuf.clone(), ChainItem::new(*rel));
				}
			}
		}
	}

	/// Generates text of given length.
	/// First state is choosen randomly.
	///
	/// Returns `None` if there is no state.
	pub fn generate(&self, n: usize, rng: &mut impl RngCore) -> Option<String> {
		let mut res = String::new();

		let mut prev = Vec::with_capacity(self.state_size);
		for _ in 0..n {
			let next_spur = self.next_step(&prev, rng)?;
			let next = self.cache.resolve(&next_spur);

			res.push_str(next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next_spur);
		}

		res.pop();

		Some(res)
	}

	/// Generates text of given length, with accordance to the given starting value.
	///
	/// Returns `None` if there is no state.
	pub fn generate_start(
		&self,
		start: &str,
		n: usize,
		rng: &mut impl RngCore,
	) -> Option<String> {
		let mut res = String::new();

		let mut prev: Vec<Spur> = self
			.regex
			.find_iter(start)
			.map(|m| m.as_str())
			.collect::<Vec<&str>>()
			.into_iter()
			.rev()
			.take(2)
			.rev()
			.filter_map(|t| self.cache.get(t))
			.collect();

		for _ in 0..n {
			let next_spur = self.next_step(&prev, rng)?;
			let next = self.cache.resolve(&next_spur);

			res.push_str(next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next_spur);
		}
		res.pop();

		Some(res)
	}

	/// Returns the number of states the chain has.
	#[inline]
	pub fn len(&self) -> usize {
		self.items.len()
	}

	/// Returns the number of string that are interned in cache.
	#[inline]
	pub fn cache_len(&self) -> usize {
		self.cache.len()
	}

	/// Returns if the chain is empty or not.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.items.len() == 0
	}

	/// Returns the state size.
	#[inline]
	pub fn state_size(&self) -> usize {
		self.state_size
	}

	/// Returns a copy of the regex.
	#[inline]
	pub fn regex(&self) -> Regex {
		self.regex.clone()
	}

	/// Returns the appropriate next step for the given previous state.
	///
	/// Returns `None` if there is no state.
	fn next_step(&self, prev: &[Spur], rng: &mut impl RngCore) -> Option<Spur> {
		for i in 0..prev.len() {
			let pslice = &prev[i..];

			if let Some(res) = self.items.get(pslice) {
				return res.get_rand(rng);
			} else {
				continue;
			}
		}

		self.items
			.values()
			.collect::<Vec<&ChainItem>>()
			.choose(rng)?
			.get_rand(rng)
	}
}

/// Wrapper for Vec<Spur> to make some operations easier.
#[cfg_attr(
	feature = "serialize",
	derive(Serialize, Deserialize),
	serde(transparent)
)]
#[derive(Clone)]
struct ChainItem {
	items: Vec<Spur>,
}

impl ChainItem {
	/// Creates a ChainItem, which will also contain `s`.
	fn new(s: Spur) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	/// Adds item.
	fn add(&mut self, s: Spur) {
		self.items.push(s);
	}

	/// Gets a random item.
	fn get_rand(&self, rng: &mut impl RngCore) -> Option<Spur> {
		let res = *self
			.items
			// get a random item from the Vec
			.choose(rng)?;

		Some(res)
	}
}

/// Recommended Regex for general use.
pub static WORD_REGEX: &str = r"(\p{Alphabetic}|\d)(\p{Alphabetic}|\d|'|-)*(\.|!|\?)?";
