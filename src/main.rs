use std::{
	collections::HashMap,
	env,
	fs::{self, read_to_string},
};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use ustr::{ustr, Ustr};

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
		// Then merges them
		.fold(HashMap::new(), |a, s| gen_chain(a, s));

	// Generation
	// ~~ indicate flag
	let mut prev = ustr("~~START");
	let mut res = String::new();
	for _ in 0..10 {
		let next = markov_chain[&prev].get_rand();
		res.push_str(&next);
		res.push(' ');
		prev = next.into();
	}
	res.pop();

	println!("{}", res);
}

/// Generates Markov Chain from given string
fn gen_chain(mut mc: HashMap<Ustr, ChainItem>, s: String) -> HashMap<Ustr, ChainItem> {
	// Regex for kind of tokens we want to match.
	// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
	static WORD_REGEX: Lazy<Regex> =
		Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

	let tokens = WORD_REGEX.find_iter(&s);

	// ~~ indicate flag
	let mut prev = ustr("~~START");
	for t in tokens {
		// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
		let t = ustr(t.as_str());

		mc.entry(prev)
			.and_modify(|ci| ci.add(t))
			.or_insert(ChainItem::new(t.clone()));

		prev = t;
	}

	mc
}
