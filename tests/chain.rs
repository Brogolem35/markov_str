use markov_str::*;
use rand::SeedableRng;
use regex::Regex;

const TEST_TEXT: &str = "Hey guys, did you know that Vaporeon can learn Mist in Yellow, but only under a very specific circumstance? In Yellow, Vaporeon is meant to learn both Haze and Mist at level 42. However, the programming at the time is so bad it's impossible for a Pokémon to learn two moves at the same level. As a result, Vaporeon will only learn Haze and not Mist. Pokémon who leveled up using the Daycare do not have this restriction though. If Vaporeon reaches level 42 while in the Daycare, it will learn both Haze and Mist.";

#[test]
fn seed1() {
	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng = rand::rngs::StdRng::seed_from_u64(1337);

	assert_eq!(
		chain.generate(10, &mut rng),
		Some("impossible for a Pokémon to learn two moves at the".to_string())
	)
}

#[test]
fn seed2() {
	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	assert_eq!(chain.generate(10, &mut rng1), chain.generate(10, &mut rng2))
}

#[test]
fn clone() {
	let mut chain1 = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain1.add_text(TEST_TEXT);
	let chain2 = chain1.clone();

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	assert_eq!(
		chain1.generate(10, &mut rng1),
		chain2.generate(10, &mut rng2)
	)
}

#[test]
fn iter1() {
	const LEN: usize = 10;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate(LEN, &mut rng1).unwrap(),
			chain.iter(LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter2() {
	const LEN: usize = 25;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate(LEN, &mut rng1).unwrap(),
			chain.iter(LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter3() {
	const LEN: usize = 100;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate(LEN, &mut rng1).unwrap(),
			chain.iter(LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter_start1() {
	const LEN: usize = 10;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate_start("Vaporeon", LEN, &mut rng1).unwrap(),
			chain.iter_start("Vaporeon", LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter_start2() {
	const LEN: usize = 25;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate_start("Vaporeon", LEN, &mut rng1).unwrap(),
			chain.iter_start("Vaporeon", LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter_start3() {
	const LEN: usize = 100;

	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate_start("Vaporeon", LEN, &mut rng1).unwrap(),
			chain.iter_start("Vaporeon", LEN, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[test]
fn iter_start() {
	let mut chain = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain.add_text(TEST_TEXT);

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	for _ in 0..10 {
		assert_eq!(
			chain.generate(10, &mut rng1).unwrap(),
			chain.iter(10, &mut rng2).collect::<Vec<String>>().join(" ")
		)
	}
}

#[cfg(feature = "serialize")]
#[test]
fn serde() {
	let mut chain1 = MarkovChain::new(2, Regex::new(WORD_REGEX).unwrap());
	chain1.add_text(TEST_TEXT);
	let chain2 = chain1.clone();

	let mut rng1 = rand::rngs::StdRng::seed_from_u64(1337);
	let mut rng2 = rand::rngs::StdRng::seed_from_u64(1337);

	assert_eq!(
		chain1.generate(10, &mut rng1),
		chain2.generate(10, &mut rng2)
	)
}
