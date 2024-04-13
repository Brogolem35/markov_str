use std::{
	collections::HashMap,
	env,
	fs::{self, read_to_string},
};

use once_cell::sync::Lazy;
use regex::Regex;

static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

struct ChainItem {
	items: HashMap<String, u32>,
	totalcnt: u32,
}

impl ChainItem {
	fn new(s: String) -> ChainItem {
		let mut res = ChainItem {
			items: HashMap::new(),
			totalcnt: 1,
		};
		res.items.insert(s, 1);

		res
	}

	fn increment(&mut self, s: String) {
		self.items.entry(s).and_modify(|e| *e += 1).or_insert(1);
		self.totalcnt += 1;
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

	let contents = files
		.filter_map(|f| read_to_string(f.path()).ok())
		.map(|f| gen_chain(f))
		.collect::<Vec<HashMap<String, ChainItem>>>();
}

fn gen_chain(s: String) -> HashMap<String, ChainItem> {
	let mut mc: HashMap<String, ChainItem> = HashMap::new();

	let tokens = WORD_REGEX.find_iter(&s);

	// ~~ indicate flag
	let mut prev = String::from("~~START");
	for t in tokens {
		let t = t.as_str().to_string();

		mc.entry(prev.clone())
			.and_modify(|ci| ci.increment(t.clone()))
			.or_insert(ChainItem::new(t.clone()));

		prev = t.clone();
	}

	for (k, v) in &mc {
		println!("{}={}", k, v.items.get("ill").unwrap_or(&0));
	}

	mc
}
