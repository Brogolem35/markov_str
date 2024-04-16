use std::{
	collections::HashMap,
	env,
	fs::{self, read_to_string},
};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use ustr::{ustr, Ustr};

struct MarkovChain {
	items: HashMap<String, ChainItem>,
	state_size: usize,
}

impl MarkovChain {
	fn new(state_size: usize) -> MarkovChain {
		MarkovChain {
			items: HashMap::new(),
			state_size,
		}
	}

	/// Generates Markov Chain from given string
	fn add_text(&mut self, text: &str) {
		// Regex for kind of tokens we want to match.
		// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
		static WORD_REGEX: Lazy<Regex> =
			Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

		let tokens = WORD_REGEX.find_iter(text);

		// ~~ indicate flag
		let mut prev = Vec::with_capacity(self.state_size);
		prev.push("~~START");
		for t in tokens {
			let pstr = prev.join(" ");
			// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
			let t = ustr(t.as_str());

			if let Some(ci) = self.items.get_mut(&pstr) {
				ci.add(t);
			} else {
				self.items.insert(pstr, ChainItem::new(t.clone()));
			}

			prev.push(t.as_str());
			if prev.len() > self.state_size {
				prev.remove(0);
			}
		}
	}

	fn generate_text(&self, n: usize) -> String {
		let mut res = String::new();

		// ~~ indicate flag
		let mut prev = Vec::with_capacity(self.state_size);
		prev.push("~~START");
		for _ in 0..n {
			let pstr = prev.join(" ");

			let next = self.items[&pstr].get_rand();
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
struct ChainItem {
	items: Vec<Ustr>,
}

impl ChainItem {
	fn new(s: Ustr) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	fn add(&mut self, s: Ustr) {
		self.items.push(s);
	}

	fn get_rand(&self) -> Ustr {
		self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
			.clone()
	}
}

fn main() {
	let home_dir = env::var("HOME").expect("HOME Environment Variable not found");
	let training_path = home_dir + "/markov_chain" + "/training";

	// Gets the paths of evey file and directory in the training_path.
	let tpaths = fs::read_dir(&training_path)
		.expect(&format!("Can't read files from: {}", training_path));

	// Only the files remain
	let files = tpaths
		.filter_map(|f| f.ok())
		.filter(|f| match f.file_type() {
			Err(_) => false,
			Ok(f) => f.is_file(),
		});

	// Reads every file into a string
	let contents = files.filter_map(|f| read_to_string(f.path()).ok());

	let markov_chain = contents
		// Then merges them
		.fold(MarkovChain::new(2), |mut a, s| {
			a.add_text(&s);
			a
		});

	// Generation
	println!("{}", markov_chain.generate_text(25));
	println!("{}", markov_chain.generate_text(25));
	println!("{}", markov_chain.generate_text(25));
	println!("{}", markov_chain.generate_text(25));
	println!("{}", markov_chain.generate_text(25));
}
