use std::{
	collections::HashMap,
	env,
	fmt::format,
	fs::{self, read_to_string},
};

use regex::Regex;

struct ChainItem {
	items: Vec<(String, u32)>,
	totalcnt: u32,
}

fn main() {
	let word_regex = Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").expect("Invalid Regular Expression");

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

	for s in word_regex.find_iter(&contents.collect::<Vec<String>>()[0]) {
		println!("{}", s.as_str());
	}
}
