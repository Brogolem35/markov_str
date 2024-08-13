//! Fast and memory efficient Markov Chain implementation, optimized for text generation
//! 
//! Example
//! -------
//! 
//! ```rust
//! let training_path = "data";
//! 
//! // Gets the paths of evey file and directory in the training_path.
//! let tpaths = fs::read_dir(training_path)
//! 	.unwrap_or_else(|_| panic!("Can't read files from: {}", training_path));
//! 
//! // Only the files remain
//! let files = tpaths
//! 	.filter_map(|f| f.ok())
//! 	.filter(|f| match f.file_type() {
//! 		Err(_) => false,
//! 		Ok(f) => f.is_file(),
//! 	});
//! 
//! // Reads every file into a string
//! let contents = files.filter_map(|f| read_to_string(f.path()).ok());
//! 
//! // Creating the Markov Chain
//! let markov_chain = contents.fold(
//! 	MarkovChain::with_capacity(2, 8_000_000, Regex::new(WORD_REGEX).unwrap()),
//! 	|mut a, s| {
//! 		a.add_text(&s);
//! 		a
//! 	},
//! );
//! 
//! // Number of tokens
//! println!("{}", markov_chain.len());
//! 
//! // Generation
//! for _ in 0..10 {
//! 	println!("{}", markov_chain.generate_start("among the       ", 25));
//! }
//! ```
//! 
//! This example is taken from the `src/main.rs`, you can run it by:
//! ```
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
use rand::seq::SliceRandom;
use regex::Regex;

/// Represents a Markov Chain that is designed to generate text.
///
/// [Wikipedia](https://en.wikipedia.org/wiki/Markov_chain)
pub struct MarkovChain {
	items: HashMap<String, ChainItem>,
	state_size: usize,
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
			items: HashMap::<String, ChainItem>::new(),
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
		let tokens: Vec<_> = self.regex.find_iter(text).collect();

		let mut prev_buf: String = String::with_capacity(255);
		for t in tokens.windows(tokens.len().min(self.state_size + 1)) {
			let rel = self.cache.get_or_intern(t.last().unwrap().as_str());

			for i in 1..t.len() {
				prev_buf.clear();
				for (i, s) in t.iter().rev().skip(1).take(i).rev().enumerate() {
					if i > 0 {
						prev_buf.push(' ')
					}

					prev_buf.push_str(s.as_str());
				}

				if let Some(ci) = self.items.get_mut(&prev_buf) {
					ci.add(rel);
				} else {
					self.items.insert(prev_buf.clone(), ChainItem::new(rel));
				}
			}
		}
	}

	/// Returns the appropriate next step for the given previous state.
	pub fn next_step(&self, prev: &[&str]) -> Spur {
		for i in 0..prev.len() {
			let pslice = &prev[i..];

			let pstr = pslice.join(" ");

			if let Some(res) = self.items.get(&pstr) {
				return res.get_rand();
			} else {
				continue;
			}
		}

		self.items
			.values()
			.collect::<Vec<&ChainItem>>()
			.choose(&mut rand::thread_rng())
			.unwrap()
			.get_rand()
	}

	/// Generates text of given length.
	///
	/// First state is choosen randomly.
	pub fn generate(&self, n: usize) -> String {
		let mut res = String::new();

		let mut prev = Vec::with_capacity(self.state_size);
		for _ in 0..n {
			let next = self.next_step(&prev);
			let next = self.cache.resolve(&next);

			res.push_str(next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next);
		}

		res.pop();

		res
	}

	/// Generates text of given length, with accordance to the given starting value.
	pub fn generate_start(&self, start: &str, n: usize) -> String {
		let mut res = String::new();

		let mut prev: Vec<&str> = self.regex
			.find_iter(start)
			.map(|m| m.as_str())
			.collect::<Vec<&str>>()
			.into_iter()
			.rev()
			.take(2)
			.rev()
			.collect();

		for _ in 0..n {
			let next = self.next_step(&prev);
			let next = self.cache.resolve(&next);

			res.push_str(next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next);
		}
		res.pop();

		res
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
}

/// Wrapper for Vec<Spur> to make some operations easier.
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
	fn get_rand(&self) -> Spur {
		*self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
	}
}

/// Recommended Regex for general use.
pub static WORD_REGEX: &str = r"(\p{Alphabetic}|\d)(\p{Alphabetic}|\d|'|-)*(\.|!|\?)?";

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn regex1() {
		let rres: Vec<_> = Regex::new(WORD_REGEX)
			.unwrap()
			.find_iter("lorem ipsum dolor")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lorem", "ipsum", "dolor"]);
	}

	#[test]
	fn regex2() {
		let rres: Vec<_> = Regex::new(WORD_REGEX)
			.unwrap()
			.find_iter("lor.em ips!um 'dolor")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lor", "em", "ips", "um", "dolor"]);
	}

	#[test]
	fn regex3() {
		let rres: Vec<_> = Regex::new(WORD_REGEX)
			.unwrap()
			.find_iter("lorem ipsum dol_3or")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lorem", "ipsum", "dol", "3or"]);
	}

	#[test]
	fn regex4() {
		let rres: Vec<_> = Regex::new(WORD_REGEX)
			.unwrap()
			.find_iter("123  1,23 1_2 2d3")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["123", "1", "23", "1", "2", "2d3"]);
	}

	#[test]
	fn regex5() {
		let rres: Vec<_> = Regex::new(WORD_REGEX)
			.unwrap()
			.find_iter("ömür ğğğ 式 2d3")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["ömür", "ğğğ", "式", "2d3"]);
	}
}
