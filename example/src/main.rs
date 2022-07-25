#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use clap::Parser;
use cli_batteries::version;
use std::{io::Result, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, instrument, span, Level};

#[derive(Clone, Debug, Parser)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

#[instrument]
async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file).await?;
    {
        let span = span!(Level::INFO, "Reading file");
        let _ = span.enter();

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        event!(Level::INFO, length = contents.len(), "Read file");
    }
    Ok(())
}

fn main() {
    cli_batteries::run(version!(), app);
}
