#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use clap::Parser;
use cli_batteries::version;
use opentelemetry::{global, global::get_text_map_propagator, Context};
use std::{collections::HashMap, io::Result, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, info_span, instrument, Instrument, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber;

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

    // Extract context from request headers
    let parent_context = get_text_map_propagator(|propagator| {
        propagator.extract(&HashMap::from([(
            "traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
        )]))
    });

    let span = info_span!("Reading file", file=?options.file);
    span.set_parent(parent_context);

    file.read_to_string(&mut contents).instrument(span).await?;
    event!(Level::INFO, length = contents.len(), "Read file");
    Ok(())
}

fn main() {
    cli_batteries::run(version!(mio), app);
}
