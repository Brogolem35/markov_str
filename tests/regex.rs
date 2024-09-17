use markov_str::WORD_REGEX;
use regex::Regex;

#[test]
fn regex1() {
	let rres: Vec<_> = Regex::new(WORD_REGEX)
		.unwrap()
		.find_iter("lorem ipsum dolor")
		.map(|m| m.as_str())
		.collect();

	assert_eq!(rres, vec!["lorem", "ipsum", "dolor"]);
}

#[test]
fn regex2() {
	let rres: Vec<_> = Regex::new(WORD_REGEX)
		.unwrap()
		.find_iter("lor.em ips!um 'dolor")
		.map(|m| m.as_str())
		.collect();

	assert_eq!(rres, vec!["lor.", "em", "ips!", "um", "dolor"]);
}

#[test]
fn regex3() {
	let rres: Vec<_> = Regex::new(WORD_REGEX)
		.unwrap()
		.find_iter("lorem ipsum dol_3or")
		.map(|m| m.as_str())
		.collect();

	assert_eq!(rres, vec!["lorem", "ipsum", "dol", "3or"]);
}

#[test]
fn regex4() {
	let rres: Vec<_> = Regex::new(WORD_REGEX)
		.unwrap()
		.find_iter("123  1,23 1_2 2d3")
		.map(|m| m.as_str())
		.collect();

	assert_eq!(rres, vec!["123", "1", "23", "1", "2", "2d3"]);
}

#[test]
fn regex5() {
	let rres: Vec<_> = Regex::new(WORD_REGEX)
		.unwrap()
		.find_iter("ömür ğğğ 式 2d3")
		.map(|m| m.as_str())
		.collect();

	assert_eq!(rres, vec!["ömür", "ğğğ", "式", "2d3"]);
}
