# Changelog

## 0.3.0

- `MarkovChain` struct is renamed to `RawMarkovChain` and `MarkovChain` is a type alias for `RawMarkovChain<4>`. This shouldn't change usage.
- New `RawMarkovChain::iter*` methods.
- New `RawMarkovChain::add_text_weighted` method.
- More efficient memory use for states where sizes of the states are lesser than or equal to N, where `RawMarkovChain<N>`.

## 0.2.0

- **BREAKING:** MarkovChain::generate and MarkovChain::generate_start functions now take RngCore instead of using rand::thread_rng().
- Serialization and deserialization with [serde](https://docs.rs/serde/latest/serde/), requires `serialize` feature flag.
- MarkovChain now implements `Clone` trait.

## 0.1.0

- Initial release