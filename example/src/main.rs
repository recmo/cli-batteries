#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use clap::Parser;
use cli_batteries::{trace_from_headers, trace_to_headers, version};
use http::header::HeaderMap;
use std::{io::Result, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, info, info_span, instrument, Instrument, Level};

#[derive(Clone, Debug, Parser)]
#[group(skip)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}

#[instrument()]
async fn foobar() {
    info!("foobar called");

    let mut headers = HeaderMap::new();
    trace_to_headers(&mut headers);
    info!(?headers, "headers");
}

#[instrument(name = "Example app")]
async fn app(options: Options) -> Result<()> {
    // Pretend we are in a request
    let mut headers = HeaderMap::new();
    headers.insert(
        "traceparent",
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
            .try_into()
            .unwrap(),
    );
    trace_from_headers(&headers);

    info!(file=?options.file, "Opening file");
    let mut file = File::open(options.file.clone()).await?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .instrument(info_span!("Reading file", file=?options.file))
        .await?;

    foobar().await;

    event!(Level::INFO, length = contents.len(), "Read file");
    Ok(())
}

fn main() {
    cli_batteries::run(version!(mio), app);
}
