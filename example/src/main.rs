#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use clap::Parser;
use cli_batteries::version;
use std::{io::Result, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, info_span, instrument, Instrument, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opentelemetry::{global, Context};
use std::collections::HashMap;
use tracing_subscriber;

#[derive(Clone, Debug, Parser)]
struct Options {
    /// File to read
    #[clap(long, env, default_value = "Readme.md")]
    file: PathBuf,
}


fn make_request(_cx: Context) {
    // perform external request after injecting context
    // e.g. if there are request headers that impl `opentelemetry::propagation::Injector`
    // then `propagator.inject_context(cx, request.headers_mut())`
}

fn build_example_carrier() -> HashMap<String, String> {
    let mut carrier = HashMap::new();
    carrier.insert(
        "traceparent".to_string(),
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
    );

    carrier
}


#[instrument(name = "Example app")]
async fn app(options: Options) -> Result<()> {
    let mut file = File::open(options.file.clone()).await?;
    let mut contents = String::new();

        // Extract context from request headers
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&build_example_carrier())
        });

    let span = info_span!("Reading file", file=?options.file);
    span.set_parent( parent_context);

    file.read_to_string(&mut contents)
        .instrument(span)
        .await?;
    event!(Level::INFO, length = contents.len(), "Read file");
    Ok(())
}

fn main() {
    cli_batteries::run(version!(mio), app);
}
