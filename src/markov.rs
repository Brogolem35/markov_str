use std::collections::VecDeque;

use hashbrown::HashMap;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use ustr::{ustr, Ustr};

/// Represents a Markov Chain that is designed to generate text.
///
/// [Wikipedia](https://en.wikipedia.org/wiki/Markov_chain)
pub struct MarkovChain {
	pub items: HashMap<String, ChainItem>,
	pub state_size: usize,
}

static WORD_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r"(\p{Alphabetic}|\d)(\p{Alphabetic}|\d|'|-)*").unwrap());

impl MarkovChain {
	/// Create an empty MarkovChain.
	///
	/// The hash map of the MarkovChain is initially created with a capacity of 0, so it will not allocate until it
	/// is first inserted into.
	#[allow(dead_code)]
	pub fn new(state_size: usize) -> MarkovChain {
		MarkovChain {
			items: HashMap::<String, ChainItem>::new(),
			state_size,
		}
	}

	/// Create an empty `HashMap` with the specified capacity.
	///
	/// The hash map of the MarkovChain will be able to hold at least `capacity` elements without
	/// reallocating. If `capacity` is 0, the hash map will not allocate.
	pub fn with_capacity(state_size: usize, capacity: usize) -> MarkovChain {
		MarkovChain {
			items: HashMap::with_capacity(capacity),
			state_size,
		}
	}

	/// Add text as training data.
	pub fn add_text(&mut self, text: &str) {
		let tokens = WORD_REGEX.find_iter(text);

		let mut prev: VecDeque<&str> = VecDeque::with_capacity(self.state_size + 100);
		let mut prev_buf: String = String::with_capacity(255);
		for t in tokens {
			for i in 1..=prev.len() {
				prev_buf.clear();
				for (i, s) in prev.iter().rev().take(i).rev().enumerate() {
					if i > 0 {
						prev_buf.push(' ')
					}

					prev_buf.push_str(s);
				}

				let t = ustr(t.as_str());

				if let Some(ci) = self.items.get_mut(&prev_buf) {
					ci.add(t);
				} else {
					self.items.insert(prev_buf.clone(), ChainItem::new(t));
				}
			}

			if prev.len() == self.state_size {
				prev.pop_front();
			}
			prev.push_back(t.as_str());
		}
	}

	/// Return an appropriate next step for the previous state.
	pub fn next_step(&self, prev: &[&str]) -> Ustr {
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

			res.push_str(&next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next.as_str());
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
			res.push_str(&next);
			res.push(' ');

			if prev.len() == self.state_size {
				prev.remove(0);
			}
			prev.push(next.as_str());
		}
		res.pop();

		res
	}
}

/// Wrapper for Vec<Ustr> to make some operations easier.
pub struct ChainItem {
	pub items: Vec<Ustr>,
}

impl ChainItem {
	/// Create a ChainItem, which will also contain `s`.
	pub fn new(s: Ustr) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	/// Add item.
	pub fn add(&mut self, s: Ustr) {
		self.items.push(s);
	}

	/// Get a random item.
	pub fn get_rand(&self) -> Ustr {
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
