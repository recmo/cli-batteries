#![cfg(feature = "rand")]
use clap::Parser;
use rand::{rngs::OsRng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::num::ParseIntError;
use tracing::info;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Parser)]
#[group(skip)]
pub struct Options {
    /// Random seed for deterministic runs.
    /// If not specified a new seed is generated from OS entropy.
    #[clap(long, env, value_parser = parse_hex_u64)]
    random_seed: Option<u64>,
}

impl Options {
    pub fn init(&self) {
        // Initialize randomness source
        let rng_seed = self
            .random_seed
            .unwrap_or_else(|| OsRng::default().next_u64());
        info!("Using random seed {rng_seed:016x}");
        let _rng = ChaCha8Rng::seed_from_u64(rng_seed);
        // TODO: Use `rng` to create deterministic runs
    }
}

fn parse_hex_u64(src: &str) -> Result<u64, ParseIntError> {
    let src = src.strip_prefix("0x").unwrap_or(src);
    u64::from_str_radix(src, 16)
}
