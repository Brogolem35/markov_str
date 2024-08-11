use hashbrown::HashMap;
use lasso::{Capacity, Rodeo, Spur};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;

/// Represents a Markov Chain that is designed to generate text.
///
/// [Wikipedia](https://en.wikipedia.org/wiki/Markov_chain)
pub struct MarkovChain {
	pub items: HashMap<String, ChainItem>,
	pub state_size: usize,
	regex: Regex,
	pub cache: Rodeo,
}

pub static WORD_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r"(\p{Alphabetic}|\d)(\p{Alphabetic}|\d|'|-)*").unwrap());

impl MarkovChain {
	/// Create an empty MarkovChain.
	///
	/// The hash map of the MarkovChain is initially created with a capacity of 0, so it will not allocate until it
	/// is first inserted into.
	#[allow(dead_code)]
	pub fn new(state_size: usize, regex: Regex) -> MarkovChain {
		MarkovChain {
			items: HashMap::<String, ChainItem>::new(),
			state_size,
			regex,
			cache: Rodeo::new(),
		}
	}

	/// Create an empty `HashMap` with the specified capacity.
	///
	/// The hash map of the MarkovChain will be able to hold at least `capacity` elements without
	/// reallocating. If `capacity` is 0, the hash map will not allocate.
	pub fn with_capacity(state_size: usize, capacity: usize, regex: Regex) -> MarkovChain {
		MarkovChain {
			items: HashMap::with_capacity(capacity),
			state_size,
			regex,
			cache: Rodeo::with_capacity(Capacity::for_strings(capacity)),
		}
	}

	/// Add text as training data.
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

	/// Return an appropriate next step for the previous state.
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

	/// Generate text of given length.
	///
	/// First state is choosen randomly.
	#[allow(dead_code)]
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

	/// Generate text of given length, with accordance to the given starting value.
	pub fn generate_start(&self, start: &str, n: usize) -> String {
		let mut res = String::new();

		let mut prev: Vec<&str> = WORD_REGEX
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

			res.push_str(&next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next);
		}
		res.pop();

		res
	}
}

/// Wrapper for Vec<Ustr> to make some operations easier.
pub struct ChainItem {
	pub items: Vec<Spur>,
}

impl ChainItem {
	/// Create a ChainItem, which will also contain `s`.
	pub fn new(s: Spur) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	/// Add item.
	pub fn add(&mut self, s: Spur) {
		self.items.push(s);
	}

	/// Get a random item.
	pub fn get_rand(&self) -> Spur {
		*self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn regex1() {
		let rres: Vec<_> = WORD_REGEX
			.find_iter("lorem ipsum dolor")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lorem", "ipsum", "dolor"]);
	}

	#[test]
	fn regex2() {
		let rres: Vec<_> = WORD_REGEX
			.find_iter("lor.em ips!um 'dolor")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lor", "em", "ips", "um", "dolor"]);
	}

	#[test]
	fn regex3() {
		let rres: Vec<_> = WORD_REGEX
			.find_iter("lorem ipsum dol_3or")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["lorem", "ipsum", "dol", "3or"]);
	}

	#[test]
	fn regex4() {
		let rres: Vec<_> = WORD_REGEX
			.find_iter("123  1,23 1_2 2d3")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["123", "1", "23", "1", "2", "2d3"]);
	}

	#[test]
	fn regex5() {
		let rres: Vec<_> = WORD_REGEX
			.find_iter("ömür ğğğ 式 2d3")
			.map(|m| m.as_str())
			.collect();

		assert_eq!(rres, vec!["ömür", "ğğğ", "式", "2d3"]);
	}
}
