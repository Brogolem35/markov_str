use hashbrown::HashMap;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use ustr::{ustr, Ustr};

pub struct MarkovChain {
	pub items: HashMap<String, ChainItem>,
	pub state_size: usize,
}

impl MarkovChain {
	#[allow(dead_code)]
	pub fn new(state_size: usize) -> MarkovChain {
		MarkovChain {
			items: HashMap::<String, ChainItem>::new(),
			state_size,
		}
	}

	pub fn with_capacity(state_size: usize, capacity: usize) -> MarkovChain {
		MarkovChain {
			items: HashMap::with_capacity(capacity),
			state_size,
		}
	}

	/// Generates Markov Chain from given string
	pub fn add_text(&mut self, text: &str) {
		// Regex for kind of tokens we want to match.
		// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
		static WORD_REGEX: Lazy<Regex> =
			Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

		let tokens = find_words(text);

		// ~~ indicate flag
		let mut prev = Vec::with_capacity(self.state_size + 1);
		prev.push("~~START");
		for t in tokens.iter() {
			for i in 1..=prev.len() {
				let pslice = &prev[(prev.len() - i)..];

				let pstr = pslice.join(" ");

				// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
				let t = ustr(t.as_str());

				if let Some(ci) = self.items.get_mut(&pstr) {
					ci.add(t);
				} else {
					self.items.insert(pstr, ChainItem::new(t));
				}
			}

			prev.push(t.as_str());
			if prev.len() > self.state_size {
				prev.remove(0);
			}
		}
	}

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

	#[allow(dead_code)]
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
		*self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
	}
}

// (\w|\d|'|-)+(\.|!|\?)*
fn find_words<'a>(text: &'a str) -> Vec<String> {
	let mut buf: Vec<char> = Vec::new();
	let mut res: Vec<String> = Vec::new();
	let mut in_word = false;
	let mut post_word = false;

	for c in text.chars() {
		if !post_word && (c.is_alphanumeric() || c == '\'' || c == '-') {
			buf.push(c);
			in_word = true;
		} else if in_word {
			if c == '.' || c == '!' || c == '?' {
				buf.push(c);
				post_word = true;
			} else {
				res.push(buf.iter().collect());
				buf.clear();
				in_word = false;
				post_word = false;

				if c.is_alphanumeric() || c == '\'' || c == '-' {
					buf.push(c);
					in_word = true;
				}
			}
		}
	}

	if !buf.is_empty() {
		res.push(buf.iter().collect());
	}

	return res;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parser() {
		let word_regex = Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap();

		let text = "asfd asa... lmao.lorem swag!!?? kek     	p bla,bla ğ!.?ü";

		assert_eq!(
			find_words(text),
			word_regex
				.find_iter(text)
				.map(|m| m.as_str().to_string())
				.collect::<Vec<String>>()
		);
	}
}
