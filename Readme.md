# CLI Batteries

[![crates.io](https://buildstats.info/crate/yul)](https://crates.io/crates/yul)
[![docs.rs](https://img.shields.io/docsrs/yul)](https://docs.rs/yul)
[![MIT License](https://img.shields.io/github/license/recmo/yul)](https://github.com/recmo/yul/blob/main/mit-license.md)
[![dependency status](https://deps.rs/repo/github/recmo/yul/status.svg)](https://deps.rs/repo/github/recmo/yul)
[![codecov](https://codecov.io/gh/recmo/yul/branch/main/graph/badge.svg?token=WBPZ9U4TTO)](https://codecov.io/gh/recmo/yul)
[![CI](https://github.com/recmo/yul/actions/workflows/ci.yml/badge.svg)](https://github.com/recmo/yul/actions/workflows/ci.yml)

Opinionated batteries-included command line interface.

```rust
use cli_batteries::StructOpt;
use std::{path::PathBuf, io::Result};
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
    cli_batteries::run("my_app v0.1.0", app);
}
```

## Generate build info

```rust,ignore
// build.rs
fn main() {
    cli_batteries::build_rs();
}
```

```rust,ignore
// main.rs
fn main() {
    cli_batteries::run(version!(), app);
}
```



## Features

* `rand`: Log and configure random seeds.
* `rayon`: Log and configure number of threads.
* `prometheus`: Start a Prometheus metrics server.
* `tokio_console`: Start a Tokio console server.

## Building and testing

Format, lint, build and test everything (I recommend creating a shell alias for this):

```sh
cargo fmt &&\
cargo clippy --all-features --all-targets &&\
cargo test --workspace --all-features --doc -- --nocapture &&\
cargo test --workspace --all-features --all-targets -- --nocapture &&\
cargo doc --workspace --all-features --no-deps
```

Run benchmarks with the provided `.cargo/config.toml` alias

```sh
cargo criterion
```

Check documentation coverage

```sh
RUSTDOCFLAGS="-Z unstable-options --show-coverage"  cargo doc --workspace --all-features --no-deps
```

## To do

Goals:

Maybe:

---

[![lines of code](https://img.shields.io/tokei/lines/github/recmo/yul)](https://github.com/recmo/yul)
[![GitHub contributors](https://img.shields.io/github/contributors/recmo/yul)](https://github.com/recmo/yul/graphs/contributors)
[![GitHub issues](https://img.shields.io/github/issues/recmo/yul)](https://github.com/recmo/yul/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/recmo/yul?label=PRs)](https://github.com/recmo/yul/pulls)
[![GitHub Repo stars](https://img.shields.io/github/stars/recmo/yul)](https://star-history.com/#recmo/yul&Date)
[![crates.io](https://img.shields.io/crates/d/yul)](https://crates.io/crates/yul)
