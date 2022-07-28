#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use clap::Parser;
use cli_batteries::version;
use std::{io::Result, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, info_span, instrument, Instrument, Level};

#[derive(Clone, Debug, Parser)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

#[instrument(name = "Example app")]
async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file.clone()).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .instrument(info_span!("Reading file", file=?options.file))
        .await?;
    event!(Level::INFO, length = contents.len(), "Read file");
    Ok(())
}

fn main() {
    cli_batteries::run(version!(mio), app);
}
