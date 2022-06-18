use crate::default_from_structopt;
use eyre::Result as EyreResult;
use opentelemetry::sdk::export::trace::stdout;
use structopt::StructOpt;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
pub struct Options {}

default_from_structopt!(Options);

impl Options {
    pub fn into_layer<S>(&self) -> EyreResult<impl Layer<S>>
    where
        S: Subscriber + for<'a> LookupSpan<'a> + Sized,
    {
        let tracer = stdout::new_pipeline().install_simple();

        let layer = OpenTelemetryLayer::new(tracer).with_tracked_inactivity(true);
        Ok(layer)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
}
