use markov_str::*;
use rand::SeedableRng;
use regex::Regex;
#[cfg(feature = "serialize")]
use serde_json;
use std::fs::{self, read_to_string};

// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

fn main() {
	let training_path = "data";

	// Gets the paths of evey file and directory in the training_path.
	let tpaths = fs::read_dir(training_path)
		.unwrap_or_else(|_| panic!("Can't read files from: {}", training_path));

	// Only the files remain
	let files = tpaths
		.filter_map(|f| f.ok())
		.filter(|f| match f.file_type() {
			Err(_) => false,
			Ok(f) => f.is_file(),
		});

	// Reads every file into a string
	let contents = files.filter_map(|f| read_to_string(f.path()).ok());

	// Creating the Markov Chain
	let markov_chain = contents.fold(
		MarkovChain::with_capacity(2, 8_000_000, Regex::new(WORD_REGEX).unwrap()),
		|mut a, s| {
			a.add_text(&s);
			a
		},
	);

	// Generation
	println!("{}", markov_chain.len());

	// ThreadRng
	for _ in 0..10 {
		println!(
			"ThreadRng: {}",
			markov_chain
				.generate_start("among the       ", 25, &mut rand::thread_rng())
				.unwrap()
		);
	}

	// StdRng with seed
	let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
	for _ in 0..10 {
		println!(
			"Seeded: {}",
			markov_chain
				.generate_start("among the       ", 25, &mut rng)
				.unwrap()
		);
	}

	#[cfg(feature = "serialize")]
	{
		let res = serde_json::to_string(&markov_chain).unwrap();
		eprintln!("{}", res);
		let m: MarkovChain = serde_json::from_str(&res).unwrap();

		let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
		println!("{}", m.len());
		for _ in 0..10 {
			println!(
				"Deserialized: {}",
				m.generate_start("among the       ", 25, &mut rng).unwrap()
			);
		}
	}
}
