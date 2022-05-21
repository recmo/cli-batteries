use cli_batteries::version;
use std::{io::Result, path::PathBuf};
use structopt::StructOpt;
use tokio::fs::File;

#[derive(StructOpt)]
struct Options {
    /// File to read
    #[structopt(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file).await?;
    Ok(())
}

fn main() {
    cli_batteries::run(version!(), app);
}
