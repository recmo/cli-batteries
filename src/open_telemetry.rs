#![cfg(feature = "opentelemetry")]
use crate::default_from_structopt;
use eyre::{eyre, Result as EyreResult};
use opentelemetry::{global, runtime::Tokio, KeyValue};
use opentelemetry::sdk::{trace::{self}, Resource};
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;
use structopt::StructOpt;
use tracing::{error, info, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{registry::LookupSpan, Layer};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
pub struct Options {
    #[structopt(long, env)]
    service_name: Option<String>,

    #[structopt(long, env)]
    env: Option<String>,

    #[structopt(long, env)]
    version: Option<String>,

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
    pub fn to_layer<S>(&self) -> EyreResult<impl Layer<S>>
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
                .with_trace_config(
                    trace::config()
                        .with_resource(
                            Resource::new(vec![
                                KeyValue::new("service.name", &self.service_name),
                                KeyValue::new("env", &self.env),
                                KeyValue::new("version", &self.version)])))
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

            // Construct a reqwest client with timeouts
            let client = reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(5))
                .build()?;

            // HACK: openetelemetry-datadog adds /v0.5/traces to the url, but
            // does not remove the final / that is present in the url after
            // url.to_string(). This causes a double `//` to appear at the
            // beginning and the datadog agent will respond with a 301 redirect
            // to remove it. When handling the redirect, the method and payload
            // of the request are lost due to a separate bug.
            let url = url.to_string();
            let trimmed = url.trim_end_matches('/');

            let tracer = new_pipeline()
                .with_service_name(&self.service_name)
                .with_version(ApiVersion::Version05)
                .with_agent_endpoint(trimmed)
                .with_http_client::<reqwest::Client>(Box::new(client))
                .install_batch(Tokio)?;

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
