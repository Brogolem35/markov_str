use hashbrown::{hash_map::RawEntryMut, HashMap};
use lasso::{Capacity, Rodeo, Spur};
use rand::{seq::SliceRandom, RngCore};
use regex::Regex;
use smallvec::SmallVec;

#[cfg(feature = "serialize")]
use {
	serde::{Deserialize, Serialize},
	serde_json_any_key::*,
};

/// Represents a Markov Chain that is designed to generate text.
///
/// States with sizes that are lesser than or equal to `N` are stored inline, thus are more performant.
/// Those of sizes that are greater are stored in a seperate [`Vec`].
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct RawMarkovChain<const N: usize> {
	#[cfg_attr(feature = "serialize", serde(with = "any_key_map"))]
	items: HashMap<SmallVec<[Spur; N]>, ChainItem, foldhash::fast::FixedState>,
	state_size: usize,
	#[cfg_attr(feature = "serialize", serde(with = "serde_regex"))]
	regex: Regex,
	cache: Rodeo,
}

/// Represents a Markov Chain that is designed to generate text.
///
/// Is a type alias for [`RawMarkovChain<4>`].
pub type MarkovChain = RawMarkovChain<4>;

impl<const N: usize> RawMarkovChain<N> {
	/// Creates an empty MarkovChain.
	///
	/// The hashmap and the cache of the MarkovChain is initially created with the capacity of 0.
	/// It will not allocate until the first insertion.
	#[inline]
	pub fn new(state_size: usize, regex: Regex) -> RawMarkovChain<N> {
		RawMarkovChain {
			items: HashMap::with_hasher(foldhash::fast::FixedState::default()),
			state_size,
			regex,
			cache: Rodeo::new(),
		}
	}

	/// Creates an empty MarkovChain with the specified capacity.
	///
	/// The hashmap and the cache of the MarkovChain will be able to hold at least `capacity` elements without
	/// reallocating. If `capacity` is 0, the hashmap will not allocate.
	#[inline]
	pub fn with_capacity(
		state_size: usize,
		capacity: usize,
		regex: Regex,
	) -> RawMarkovChain<N> {
		RawMarkovChain {
			items: HashMap::with_capacity_and_hasher(
				capacity,
				foldhash::fast::FixedState::default(),
			),
			state_size,
			regex,
			cache: Rodeo::with_capacity(Capacity::for_strings(capacity)),
		}
	}

	/// Adds text as training data. The tokens will be created with the regex of the MarkovChain.
	pub fn add_text(&mut self, text: &str) {
		let tokens: Vec<Spur> = self
			.regex
			.find_iter(text)
			.map(|t| self.cache.get_or_intern(t.as_str()))
			.collect();

		// vec.windows(0) panics for some reason.
		if tokens.is_empty() {
			return;
		}

		for win in tokens.windows(tokens.len().min(self.state_size + 1)) {
			let wlen = win.len();
			let rel = win.last().unwrap();

			// if wlen is less than 2, there is nothing to do
			for i in 2..=wlen {
				// win[(wlen - 1)] == rel == win.last()
				// this is equal to win.iter().rev().skip(1).take(i - 1).rev()
				let slice = &win[(wlen - i)..(wlen - 1)];
				match self.items.raw_entry_mut().from_key(slice) {
					RawEntryMut::Occupied(mut view) => {
						view.get_mut().add(*rel);
					}
					RawEntryMut::Vacant(view) => {
						view.insert(
							SmallVec::from_slice(slice),
							ChainItem::new(*rel),
						);
					}
				}
			}
		}
	}

	/// Adds text as training data with a weight. The tokens will be created with the regex of the MarkovChain.
	///
	/// It is mostly equivalent to calling [`MarkovChain::add_text()`] `weight` number of times, but
	/// may not yield the same results when [`MarkovChain::generate()`] is called with same RNG,
	/// due to internal workings.
	pub fn add_text_weighted(&mut self, text: &str, weight: usize) {
		if weight == 0 {
			return;
		}

		let tokens: Vec<Spur> = self
			.regex
			.find_iter(text)
			.map(|t| self.cache.get_or_intern(t.as_str()))
			.collect();

		// vec.windows(0) panics for some reason.
		if tokens.is_empty() {
			return;
		}

		for win in tokens.windows(tokens.len().min(self.state_size + 1)) {
			let wlen = win.len();
			let rel = win.last().unwrap();

			// if wlen is less than 2, there is nothing to do
			for i in 2..=wlen {
				// win[(wlen - 1)] == rel == win.last()
				// this is equal to win.iter().rev().skip(1).take(i - 1).rev()
				let slice = &win[(wlen - i)..(wlen - 1)];
				match self.items.raw_entry_mut().from_key(slice) {
					RawEntryMut::Occupied(mut view) => {
						view.get_mut().add_weighted(*rel, weight);
					}
					RawEntryMut::Vacant(view) => {
						view.insert(
							SmallVec::from_slice(slice),
							ChainItem::new_weighted(*rel, weight),
						);
					}
				}
			}
		}
	}

	/// Generates text of given length.
	/// First state is choosen randomly.
	///
	/// Returns `None` if there is no state.
	pub fn generate(&self, length: usize, rng: &mut impl RngCore) -> Option<String> {
		if self.is_empty() {
			return None;
		}

		let mut res = String::new();
		for next in self.iter(length, rng) {
			res.push_str(next);
			res.push(' ');
		}
		res.pop();

		Some(res)
	}

	/// Generates text of given length, with accordance to the given starting value.
	///
	/// Returns `None` if there is no state.
	pub fn generate_start(
		&self,
		start: &str,
		length: usize,
		rng: &mut impl RngCore,
	) -> Option<String> {
		if self.is_empty() {
			return None;
		}

		let mut res = String::new();
		for next in self.iter_start(start, length, rng) {
			res.push_str(next);
			res.push(' ');
		}
		res.pop();

		Some(res)
	}

	/// Returns the number of states the chain has.
	#[inline]
	pub fn len(&self) -> usize {
		self.items.len()
	}

	/// Returns the number of string that are interned in cache.
	#[inline]
	pub fn cache_len(&self) -> usize {
		self.cache.len()
	}

	/// Returns whether the chain is empty or not.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the state size.
	#[inline]
	pub fn state_size(&self) -> usize {
		self.state_size
	}

	/// Returns a copy of the regex.
	#[inline]
	pub fn regex(&self) -> Regex {
		self.regex.clone()
	}

	/// Does the same thing as [`MarkovChain::generate()`] but instead of returning a String, returns a lazily evaluated iterator.
	#[inline]
	pub fn iter<'a>(
		&'a self,
		count: usize,
		rng: &'a mut dyn RngCore,
	) -> MarkovChainIter<'a, N> {
		MarkovChainIter {
			chain: self,
			count,
			rng,
			prev: Vec::with_capacity(self.state_size),
		}
	}

	/// Does the same thing as [`MarkovChain::generate_start()`] but instead of returning a String, returns a lazily evaluated iterator.
	#[inline]
	pub fn iter_start<'a>(
		&'a self,
		start: &str,
		count: usize,
		rng: &'a mut dyn RngCore,
	) -> MarkovChainIter<'a, N> {
		let prev: Vec<Spur> = self
			.regex
			.find_iter(start)
			.map(|m| m.as_str())
			.collect::<Vec<&str>>()
			.into_iter()
			.rev()
			.take(self.state_size)
			.rev()
			.filter_map(|t| self.cache.get(t))
			.collect();

		MarkovChainIter {
			chain: self,
			count,
			rng,
			prev,
		}
	}

	/// Returns the appropriate next step for the given previous state.
	///
	/// Returns `None` if there is no state.
	fn next_step(&self, prev: &[Spur], rng: &mut impl RngCore) -> Option<Spur> {
		for i in 0..prev.len() {
			let pslice = &prev[i..];

			if let Some(res) = self.items.get(pslice) {
				return res.get_rand(rng);
			} else {
				continue;
			}
		}

		self.items
			.values()
			.collect::<Vec<&ChainItem>>()
			.choose(rng)?
			.get_rand(rng)
	}
}

/// Iterator that iterates over generation steps.
pub struct MarkovChainIter<'a, const N: usize> {
	chain: &'a RawMarkovChain<N>,
	count: usize,
	rng: &'a mut dyn RngCore,
	prev: Vec<Spur>,
}

impl<'a, const N: usize> Iterator for MarkovChainIter<'a, N> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		if self.count == 0 {
			return None;
		}
		self.count -= 1;

		let next_spur = self.chain.next_step(&self.prev, &mut self.rng)?;
		let next = self.chain.cache.resolve(&next_spur);

		if self.prev.len() == self.chain.state_size {
			self.prev.remove(0);
		}
		self.prev.push(next_spur);

		Some(next)
	}
}

/// Wrapper for Vec<Spur> to make some operations easier.
#[cfg_attr(
	feature = "serialize",
	derive(Serialize, Deserialize),
	serde(transparent)
)]
#[derive(Clone)]
struct ChainItem {
	items: Vec<Spur>,
}

impl ChainItem {
	/// Creates a ChainItem, which will also contain `s`.
	#[inline]
	fn new(s: Spur) -> ChainItem {
		ChainItem { items: vec![s] }
	}

	/// Creates a ChainItem, which will also contain `s` `weight` number of times.
	#[inline]
	fn new_weighted(s: Spur, weight: usize) -> ChainItem {
		ChainItem {
			items: vec![s; weight],
		}
	}

	/// Adds item.
	#[inline]
	fn add(&mut self, s: Spur) {
		self.items.push(s);
	}

	/// Adds item `weight` number of times.
	#[inline]
	fn add_weighted(&mut self, s: Spur, weight: usize) {
		self.items.extend(std::iter::repeat(s).take(weight));
	}

	/// Gets a random item.
	#[inline]
	fn get_rand(&self, rng: &mut impl RngCore) -> Option<Spur> {
		let res = *self
			.items
			// get a random item from the Vec
			.choose(rng)?;

		Some(res)
	}
}
