use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHasher};
use std::hash::BuildHasherDefault;
use ustr::{ustr, Ustr};

pub struct MarkovChain {
	pub items: FxHashMap<String, ChainItem>,
	pub state_size: usize,
}

impl MarkovChain {
	pub fn new(state_size: usize) -> MarkovChain {
		MarkovChain {
			items: FxHashMap::<String, ChainItem>::default(),
			state_size,
		}
	}

	pub fn with_capacity(state_size: usize, capacity: usize) -> MarkovChain {
		MarkovChain {
			items: FxHashMap::with_capacity_and_hasher(
				capacity,
				BuildHasherDefault::<FxHasher>::default(),
			),
			state_size,
		}
	}

	/// Generates Markov Chain from given string
	pub fn add_text(&mut self, text: &str) {
		// Regex for kind of tokens we want to match.
		// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
		static WORD_REGEX: Lazy<Regex> =
			Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

		let tokens = WORD_REGEX.find_iter(text);

		// ~~ indicate flag
		let mut prev = Vec::with_capacity(self.state_size + 1);
		prev.push("~~START");
		for t in tokens {
			for i in 1..=prev.len() {
				let pslice = &prev[(prev.len() - i)..];

				let pstr = pslice.join(" ");

				// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
				let t = ustr(t.as_str());

				if let Some(ci) = self.items.get_mut(&pstr) {
					ci.add(t);
				} else {
					self.items.insert(pstr, ChainItem::new(t.clone()));
				}
			}

			prev.push(t.as_str());
			if prev.len() > self.state_size {
				prev.remove(0);
			}
		}
	}

	pub fn next_step(&self, prev: &Vec<&str>) -> Ustr {
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

	pub fn generate(&self, n: usize) -> String {
		let mut res = String::new();

		// ~~ indicate flag
		let mut prev = Vec::with_capacity(self.state_size + 1);
		for _ in 0..n {
			let next = self.next_step(&prev);

			res.push_str(&next);
			res.push(' ');

			prev.push(next.as_str());
			if prev.len() > self.state_size {
				prev.remove(0);
			}
		}

		res.pop();

		res
	}

	pub fn generate_start(&self, start: &str, n: usize) -> String {
		static WORD_REGEX: Lazy<Regex> =
			Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

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

			prev.push(next.as_str());
			if prev.len() > self.state_size {
				prev.remove(0);
			}
		}
		res.pop();

		res
	}
}

/// Wrapper for Vec<Ustr> to make some operations easier
pub struct ChainItem {
	pub items: Vec<Ustr>,
}

impl ChainItem {
	pub fn new(s: Ustr) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	pub fn add(&mut self, s: Ustr) {
		self.items.push(s);
	}

	pub fn get_rand(&self) -> Ustr {
		self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
			.clone()
	}
}
