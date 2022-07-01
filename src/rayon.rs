#![cfg(feature = "rayon")]
use crate::default_from_clap;
use clap::Parser;
use eyre::{Result, WrapErr};
use rayon::ThreadPoolBuilder;
use tracing::info;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
pub struct Options {
    /// Number of compute threads to use. Defaults to number of cores.
    #[clap(long, env)]
    threads: Option<usize>,
}

default_from_clap!(Options);

impl Options {
    pub fn init(&self) -> Result<()> {
        let num_cpus = num_cpus::get();
        let threads = self.threads.unwrap_or(num_cpus);
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .wrap_err("Failed to build thread pool.")?;
        info!(
            "Using {} compute threads on {} cores",
            rayon::current_num_threads(),
            num_cpus
        );
        Ok(())
    }
}
