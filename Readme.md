# CLI Batteries

[![crates.io](https://buildstats.info/crate/cli-batteries)](https://crates.io/crates/cli-batteries)
[![docs.rs](https://img.shields.io/docsrs/cli-batteries)](https://docs.rs/cli-batteries)
[![MIT License](https://img.shields.io/github/license/recmo/cli-batteries)](https://github.com/recmo/cli-batteries/blob/main/mit-license.md)
[![dependency status](https://deps.rs/repo/github/recmo/cli-batteries/status.svg)](https://deps.rs/repo/github/recmo/cli-batteries)
[![codecov](https://codecov.io/gh/recmo/cli-batteries/branch/main/graph/badge.svg?token=WBPZ9U4TTO)](https://codecov.io/gh/recmo/cli-batteries)
[![CI](https://github.com/recmo/cli-batteries/actions/workflows/ci.yml/badge.svg)](https://github.com/recmo/cli-batteries/actions/workflows/ci.yml)

Opinionated batteries-included command line interface runtime utilities.

To use it, add it to your `Cargo.toml`

```toml
[dependencies]
cli-batteries = "0.1"

[build-dependencies]
cli-batteries = "0.1"
```

and call the [`build_rs`] function in your `build.rs`

```rust,ignore
fn main() {
    cli_batteries::build_rs().unwrap()
}
```

Then in your `src/main.rs` you define app specific command line arguments using [`clap::Parser`][clap] and run the app as follows

```rust,ignore
use cli_batteries::{version, Parser};
use std::{path::PathBuf, io::Result};
use tokio::fs::File;

#[derive(Parser)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file).await?;
    Ok(())
}

fn main() {
    cli_batteries::run(version!(), app);
}
```

You can see this working in the [example project](./example).

## Features

* `mimalloc`: Use the [mimalloc] allocator with security hardening features enabled.
* `rand`: Log and configure random seeds.
* `rayon`: Log and configure number of threads.
* `prometheus`: Start a Prometheus metrics server.
* `metered-allocator`: Collect metric on memory allocation, enables `prometheus`.
* `mock-shutdown`: Enable the `reset_shutdown` function that allows re-arming shutdown for testing.
* `tokio-console`: Enable the `--tokio-console` option to start a Tokio console server on `http://127.0.0.1:6669/` for async inspection.
* `otlp`: Enable the `--trace-otlp` option to push traces to an OpenTelementry collector.
* `datadog`: Enable the `--trace-datadog` option to push traces to a DataDog v5 agent.

[mimalloc]: https://github.com/microsoft/mimalloc


## Building and testing

Format, lint, build and test everything (I recommend creating a shell alias for this):

```sh
cargo fmt &&\
cargo clippy --all-features --all-targets &&\
cargo test --workspace --all-features --doc -- --nocapture &&\
cargo test --workspace --all-features --all-targets -- --nocapture &&\
cargo doc --workspace --all-features --no-deps
```

Check documentation coverage

```sh
RUSTDOCFLAGS="-Z unstable-options --show-coverage"  cargo doc --workspace --all-features --no-deps
```

## To do

Goals:

Maybe:

---

[![lines of code](https://img.shields.io/tokei/lines/github/recmo/cli-batteries)](https://github.com/recmo/cli-batteries)
[![GitHub contributors](https://img.shields.io/github/contributors/recmo/cli-batteries)](https://github.com/recmo/cli-batteries/graphs/contributors)
[![GitHub issues](https://img.shields.io/github/issues/recmo/cli-batteries)](https://github.com/recmo/cli-batteries/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/recmo/cli-batteries?label=PRs)](https://github.com/recmo/cli-batteries/pulls)
[![GitHub Repo stars](https://img.shields.io/github/stars/recmo/cli-batteries)](https://star-history.com/#recmo/cli-batteries&Date)
[![crates.io](https://img.shields.io/crates/d/cli-batteries)](https://crates.io/crates/cli-batteries)
