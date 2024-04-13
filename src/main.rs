use std::{
	collections::HashMap,
	env,
	fs::{self, read_to_string},
};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;

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
			.choose(&mut rand::thread_rng())
			.unwrap()
			.to_string()
	}
}

fn main() {
	let home_dir = env::var("HOME").expect("HOME Environment Variable not found");
	let training_path = format!("{}/{}/{}", &home_dir, "markov_chain", "training");

	let tpaths = fs::read_dir(&training_path)
		.expect(&format!("Can't read files from: {}", training_path));

	// Only the files remain
	let files = tpaths
		.filter_map(|f| f.ok())
		.filter(|f| match f.file_type() {
			Err(_) => false,
			Ok(f) => f.is_file(),
		});

	let contents = files.filter_map(|f| read_to_string(f.path()).ok());

	let markov_chain = contents
		.map(|f| gen_chain(f))
		.reduce(merge_chain)
		.expect("No chain to generate");

	let mut prev = String::from("~~START");
	for _ in 0..30 {
		let next = markov_chain[&prev].get_rand();
		print!("{} ", next);
		prev = next;
	}

	println!();
}

fn gen_chain(s: String) -> HashMap<String, ChainItem> {
	static WORD_REGEX: Lazy<Regex> =
		Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

	let mut mc: HashMap<String, ChainItem> = HashMap::new();

	let tokens = WORD_REGEX.find_iter(&s);

	// ~~ indicate flag
	let mut prev = String::from("~~START");
	for t in tokens {
		let t = t.as_str().to_string();

		mc.entry(prev.clone())
			.and_modify(|ci| ci.add(t.clone()))
			.or_insert(ChainItem::new(t.clone()));

		prev = t.clone();
	}

	mc
}

fn merge_chain(
	mut a: HashMap<String, ChainItem>,
	b: HashMap<String, ChainItem>,
) -> HashMap<String, ChainItem> {
	for (k, mut v) in b {
		a.entry(k).and_modify(|i| i.merge(&mut v)).or_insert(v);
	}

	a
}
