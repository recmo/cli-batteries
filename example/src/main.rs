#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use cli_batteries::version;
use std::{io::Result, path::PathBuf};
use clap::Parser;
use tokio::fs::File;
use tracing::instrument;

#[derive(Clone, Debug, Parser)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

#[instrument]
async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file).await?;
    Ok(())
}

fn main() {
    cli_batteries::run(version!(), app);
}
