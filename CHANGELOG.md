# Changelog

## 0.2.0

- **Breaking:** MarkovChain::generate and MarkovChain::generate_start functions now take RngCore instead of using rand::thread_rng().
- Serialization and deserialization with [serde](https://docs.rs/serde/latest/serde/), requires `serialize` feature flag.
- MarkovChain now implements `Clone` trait.

## 0.1.0

- Initial release