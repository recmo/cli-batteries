#![cfg(feature = "rayon")]
use eyre::{Result, WrapErr};
use rayon::ThreadPoolBuilder;
use structopt::StructOpt;
use tracing::info;

#[derive(Debug, PartialEq, StructOpt)]
pub struct Options {
    /// Number of compute threads to use. Defaults to number of cores.
    #[structopt(long, env)]
    threads: Option<usize>,
}

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
