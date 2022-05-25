#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
use cli_batteries::{run, default_from_structopt, Version};
use std::{io::Result, path::PathBuf};
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};

const MOCK_VERSION: Version = Version {
    pkg_name:     "cli-test",
    pkg_version:  "v0.0.0",
    pkg_repo:     "https://github.com/recmo/cli-batteries",
    crate_name:   "test",
    commit_hash:  "7cdd3615368b7e2ed1e053f33628fe7f65e6a538",
    long_version: "v0.0.0 First release",
    target:       "aarch64-apple-darwin",
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
struct Options {
    /// File to read. Note that you can't actually supply it as flag since
    /// arguments are shared with the test runner and it will reject this flag.
    #[structopt(long, env, default_value = "Readme.md")]
    file: PathBuf,

    /// Hack to make tests pass with `--nocapture`. The tests share arguments
    /// with the test runner.
    // Ideally we'd use a catch-all argument, but that doesn't seem to exist.
    #[structopt(long)]
    nocapture: bool,
}

default_from_structopt!(Options);

async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(())
}

#[test]
fn main() {
    run(MOCK_VERSION, app);
}
