#![cfg(feature = "opentelemetry")]
use crate::default_from_structopt;
use eyre::{eyre, Result as EyreResult};
use opentelemetry::{
    global,
    runtime::Tokio,
};
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;
use structopt::StructOpt;
use tracing::{error, info, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{registry::LookupSpan, Layer};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
pub struct Options {
    /// Push telemetry traces to an OpenTelemetry node.
    /// Example: grpc://localhost:4317
    #[cfg(feature = "otlp")]
    #[structopt(long, env)]
    trace_otlp: Option<Url>,

    /// Push telemetry traces to a DataDog agent. Uses api version 5.
    /// Example: http://localhost:8126
    #[cfg(feature = "datadog")]
    #[structopt(long, env)]
    trace_datadog: Option<Url>,
}

default_from_structopt!(Options);

impl Options {
    pub fn into_layer<S>(&self) -> EyreResult<impl Layer<S>>
    where
        S: Subscriber + for<'a> LookupSpan<'a> + Sized,
    {
        #[cfg(all(feature = "otlp", feature = "datadog"))]
        if self.trace_otlp.is_some() && self.trace_datadog.is_some() {
            return Err(eyre!(
                "Cannot specify both --trace-otel and --trace-datadog",
            ));
        }

        // Propagate errors in the OpenTelemetry stack to the log.
        global::set_error_handler(|error| {
            error!(%error, "Error in OpenTelemetry: {}", error);
        })?;

        #[cfg(feature = "otlp")]
        if let Some(url) = &self.trace_otlp {
            use opentelemetry_otlp::{new_exporter, new_pipeline, Protocol};

            let protocol = match url.scheme() {
                "http" => Protocol::HttpBinary,
                "grpc" => Protocol::Grpc,
                _ => {
                    return Err(eyre!(
                        "Invalid protocol: {} expecting 'http' or 'grpc'",
                        url.scheme()
                    ))
                }
            };

            let exporter = new_exporter()
                .tonic()
                .with_endpoint(url.to_string())
                .with_protocol(protocol)
                .with_timeout(Duration::from_secs(3));

            let tracer = new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .install_batch(Tokio)?;

            // See <https://docs.rs/opentelemetry-otlp/0.10.0/opentelemetry_otlp/#kitchen-sink-full-configuration>
            let layer = OpenTelemetryLayer::new(tracer).with_tracked_inactivity(true);

            return Ok(Some(layer));
        }

        #[cfg(feature = "datadog")]
        if let Some(url) = &self.trace_datadog {
            use opentelemetry_datadog::{new_pipeline, ApiVersion};

            // TODO: Early logging so we can actually see this message.
            info!(?url, "Sending traces to DataDog agent");

            // TODO: Custom reqwest client with timeout.

            let tracer = new_pipeline()
                .with_service_name("open_telemetry")
                .with_version(ApiVersion::Version05)
                .with_agent_endpoint(url.to_string())
                .install_batch(Tokio)?;

            let layer = OpenTelemetryLayer::new(tracer).with_tracked_inactivity(true);

            return Ok(Some(layer));
        }

        // Dummy layer
        return Ok(None);
    }
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}

#[cfg(test)]
pub mod test {
    use super::*;
}
