use std::{
	collections::HashMap,
	env,
	fs::{self, read_to_string},
};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;

/// Wrapper for Vec<String> to make some operations easier
struct ChainItem {
	items: Vec<String>,
}

impl ChainItem {
	fn new(s: String) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	fn add(&mut self, s: String) {
		self.items.push(s);
	}

	fn merge(&mut self, other: &mut ChainItem) {
		self.items.append(&mut other.items)
	}

	fn get_rand(&self) -> String {
		self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
			.to_string()
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
		// Generates seperate chains for every string
		.map(gen_chain)
		// Then merges them
		.reduce(merge_chain)
		.expect("No chain to generate");

	// Generation
	// ~~ indicate flag
	let mut prev = String::from("~~START");
	let mut res = String::new();
	for _ in 0..10 {
		let next = markov_chain[&prev].get_rand();
		res.push_str(&next);
		res.push(' ');
		prev = next;
	}
	res.pop();

	println!("{}", res);
}

/// Generates Markov Chain from given string
fn gen_chain(s: String) -> HashMap<String, ChainItem> {
	// Regex for kind of tokens we want to match.
	// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
	static WORD_REGEX: Lazy<Regex> =
		Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

	let mut mc: HashMap<String, ChainItem> = HashMap::new();

	let tokens = WORD_REGEX.find_iter(&s);

	// ~~ indicate flag
	let mut prev = String::from("~~START");
	for t in tokens {
		// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
		let t = t.as_str().to_string();

		mc.entry(prev.clone())
			.and_modify(|ci| ci.add(t.clone()))
			.or_insert(ChainItem::new(t.clone()));

		prev = t.clone();
	}

	mc
}

/// Merges given Markov Chains
fn merge_chain(
	mut a: HashMap<String, ChainItem>,
	b: HashMap<String, ChainItem>,
) -> HashMap<String, ChainItem> {
	for (k, mut v) in b {
		a.entry(k).and_modify(|i| i.merge(&mut v)).or_insert(v);
	}

	a
}
