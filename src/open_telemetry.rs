#![cfg(feature = "otlp")]
use crate::default_from_structopt;
use eyre::{eyre, Result as EyreResult};
use opentelemetry::{
    global,
    runtime::Tokio,
    sdk::{
        trace::{self},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource;
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
    #[structopt(long, env)]
    trace_otlp: Option<Url>,

    #[structopt(long, env)]
    trace_resource: Vec<(String, String)>,
}

default_from_structopt!(Options);

impl Options {
    pub fn to_layer<S>(&self) -> EyreResult<impl Layer<S>>
    where
        S: Subscriber + for<'a> LookupSpan<'a> + Sized,
    {
        // Propagate errors in the OpenTelemetry stack to the log.
        global::set_error_handler(|error| {
            error!(%error, "Error in OpenTelemetry: {}", error);
        })?;

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

            // Attributes for the trace generating entity.
            // See https://opentelemetry.io/docs/reference/specification/resource/semantic_conventions/
            let resource = {
                use resource::*;
                let build = Resource::new([
                    SERVICE_NAME.string(),
                    SERVICE_VERSION.string(),
                    KeyValue::new("env", "TODO"),
                    KeyValue::new("version", "TODO"),
                ]);
                let arg = Resource::empty();

                // Allow CLI to override build info.
                build.merge(arg)
            };

            let exporter = new_exporter()
                .tonic()
                .with_endpoint(url.to_string())
                .with_protocol(protocol)
                .with_timeout(Duration::from_secs(3));

            let tracer = new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(trace::config().with_resource(resource))
                .install_batch(Tokio)?;

            // See <https://docs.rs/opentelemetry-otlp/0.10.0/opentelemetry_otlp/#kitchen-sink-full-configuration>
            let layer = OpenTelemetryLayer::new(tracer).with_tracked_inactivity(true);

            return Ok(Some(layer));
        }

        // Dummy layer
        Ok(None)
    }
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
