#![cfg(feature = "otlp")]
use crate::{default_from_clap, Version};
use clap::Parser;
use eyre::{eyre, Result as EyreResult};
use heck::ToSnakeCase;
use http::header::HeaderMap;
use opentelemetry::{
    global::{self, get_text_map_propagator},
    runtime::Tokio,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{self, IdGenerator, Sampler, TracerProvider},
        Resource,
    },
    trace::TracerProvider as _,
    KeyValue,
};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_semantic_conventions::resource;
use std::{env, error::Error, str::FromStr, time::Duration};
use tracing::{error, Span, Subscriber};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::{registry::LookupSpan, Layer};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
#[group(skip)]
pub struct Options {
    /// Push telemetry traces to an OpenTelemetry node.
    /// Example: grpc://localhost:4317
    #[clap(long, env)]
    trace_otlp: Option<Url>,

    /// Attributes to set on the trace submitting entity. By default
    /// `service.name` and `service.version` are set.
    ///
    /// You can supply multiple arguments like
    /// `--trace-resource env=prod --trace-resource region=us-east-1`.
    ///
    /// They can also be set via the `TRACE_RESOURCE_*` environment variables
    /// where `*` is the attribute name converted to SHOUTY_SNAKE_CASE:
    /// `TRACE_RESOURCE_SERVICE_NAMESPACE=prod`.
    #[clap(long, value_parser = parse_key_val::<String, String>)]
    trace_resource: Vec<(String, String)>,
}

default_from_clap!(Options);

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync>>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

impl Options {
    pub fn to_layer<S>(&self, version: &Version) -> EyreResult<impl Layer<S>>
    where
        S: Subscriber + for<'a> LookupSpan<'a> + Sized + Send + Sync,
    {
        // Propagate errors in the OpenTelemetry stack to the log.
        global::set_error_handler(|error| {
            error!(%error, "Error in OpenTelemetry: {}", error);
        })?;

        // Set a format for propagating context. TraceContextPropagator implements
        // W3C Trace Context <https://www.w3.org/TR/trace-context/>
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Attributes for the trace generating entity.
        // See https://opentelemetry.io/docs/reference/specification/resource/semantic_conventions/
        let resource = {
            let build = Resource::new([
                resource::SERVICE_NAME.string(version.pkg_name),
                resource::SERVICE_VERSION
                    .string(format!("{}-{}", version.pkg_version, version.commit_hash)),
            ]);
            let env_vals = Resource::new(env::vars().filter_map(|(k, v)| {
                k.strip_prefix("TRACE_RESOURCE_")
                    .map(|k| KeyValue::new(k.to_snake_case().replace('_', "."), v))
            }));
            let cli = Resource::new(
                self.trace_resource
                    .iter()
                    .map(|(k, v)| KeyValue::new(k.clone(), v.clone())),
            );

            // Order of precedence: command line arguments, environment, build info.
            build.merge(&env_vals).merge(&cli)
        };

        let trace_config = trace::config()
            .with_sampler(Sampler::AlwaysOn)
            .with_id_generator(IdGenerator::default())
            .with_max_events_per_span(64)
            .with_max_attributes_per_span(16)
            .with_max_events_per_span(16)
            .with_resource(resource);

        if let Some(url) = &self.trace_otlp {
            use opentelemetry_otlp::{new_exporter, new_pipeline, Protocol, WithExportConfig};

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

            // See <https://docs.rs/opentelemetry-otlp/0.10.0/opentelemetry_otlp/#kitchen-sink-full-configuration>

            let exporter = new_exporter()
                .tonic()
                .with_endpoint(url.to_string())
                .with_protocol(protocol)
                .with_timeout(Duration::from_secs(3));

            let tracer = new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(trace_config)
                .install_batch(Tokio)?;

            Ok(OpenTelemetryLayer::new(tracer)
                .with_tracked_inactivity(true)
                .boxed())
        } else {
            // Create a non-exportin otel layer that produces span and trace ids for logs.
            let trace_provider = TracerProvider::builder().with_config(trace_config).build();
            let tracer = trace_provider.versioned_tracer(
                env!("CARGO_PKG_NAME"),
                Some(env!("CARGO_PKG_VERSION")),
                Some(env!("CARGO_PKG_REPOSITORY")),
            );
            let _old_provider = global::set_tracer_provider(trace_provider);
            Ok(OpenTelemetryLayer::new(tracer)
                .with_tracked_inactivity(true)
                .boxed())
        }
    }
}

/// Extract the W3C Trace Context from the headers of a request and add them
/// to the current span.
pub fn trace_from_headers(headers: &HeaderMap) {
    Span::current().set_parent(get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(headers))
    }));
}

/// Insert the W3C Trace Context to the headers of a request.
pub fn trace_to_headers(headers: &mut HeaderMap) {
    get_text_map_propagator(|propagator| {
        propagator.inject_context(&Span::current().context(), &mut HeaderInjector(headers));
    });
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
