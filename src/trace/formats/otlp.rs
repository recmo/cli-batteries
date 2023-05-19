use std::{
    fmt::{Error, Result},
    thread,
};

use chrono::Utc;
use serde::{ser::SerializeMap, Serializer};
use serde_json::Value;
use tracing::{Event, Level, Subscriber};
use tracing_serde::fields::AsMap;
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::LookupSpan,
};

use crate::trace::utils::{extract, write_adapter::WriteAdaptor};

// Implements <https://opentelemetry.io/docs/reference/specification/logs/data-model/>
// <https://opentelemetry.io/docs/reference/specification/logs/semantic_conventions/>
// <https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/>

// See https://github.com/tokio-rs/tracing/issues/1531#issuecomment-988172764

// Note that span ids can get recycled and are not up to the standards from
// OTLP. https://docs.rs/tracing-subscriber/latest/tracing_subscriber/struct.Registry.html#span-id-generation

pub struct OtlpFormatter;

#[derive(Debug)]
pub struct TraceInfo {
    pub trace_id: String,
    pub span_id:  String,
}

impl<S, N> FormatEvent<S, N> for OtlpFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    #[allow(clippy::too_many_lines)]
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> Result
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let meta = event.metadata();

        // Event metadata
        let timestamp = Utc::now().timestamp_millis();
        let (severity_text, severity_number) = match *meta.level() {
            Level::TRACE => ("TRACE", 1),
            Level::DEBUG => ("DEBUG", 5),
            Level::INFO => ("INFO", 9),
            Level::WARN => ("WARN", 13),
            Level::ERROR => ("ERROR", 17),
        };
        let mut body = String::new();
        let mut attributes = serde_json::Map::<String, Value>::new();

        let span_id = extract::opentelemetry_span_id(ctx);
        let trace_id = extract::opentelemetry_trace_id(ctx);

        // https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/span-general/#source-code-attributes
        // attributes.insert("code.function".into(), meta.target().into());
        meta.module_path()
            .map(|s| attributes.insert("code.namespace".into(), s.into()));
        if let Some(filepath) = meta.file() {
            attributes.insert("code.filepath".into(), filepath.into());
        }
        if let Some(lineno) = meta.line() {
            attributes.insert("code.lineno".into(), lineno.into());
        }

        // https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/span-general/#source-code-attributes
        // tracing-subscriber does. TODO (blocked): https://github.com/rust-lang/rust/issues/67939
        attributes.insert(
            "thread.name".into(),
            thread::current().name().map_or_else(
                || format!("{:?}", thread::current().id()).into(),
                Into::into,
            ),
        );

        // Collect event fields
        let fields = serde_json::to_value(&event.field_map()).map_err(|_| Error)?;
        if let Value::Object(map) = fields {
            attributes.extend(map.into_iter().filter_map(|(k, v)| {
                match k.as_str() {
                    // Extract `message` as `Body`
                    "message" => {
                        body = v.as_str().unwrap_or_default().to_string();
                        None
                    }
                    // Convert `log` crate fields to OpenTelemetry attributes
                    "log.file" => Some(("code.filepath".into(), v)),
                    "log.line" => Some(("code.lineno".into(), v)),
                    "log.module_path" => Some(("code.namespace".into(), v)),
                    "log.target" => None,
                    // Pass through
                    _ => Some((k, v)),
                }
            }));
        }

        // Collect span fields (if span).
        let span = if meta.is_span() {
            event.parent().and_then(|id| ctx.span(id))
        } else {
            None
        };
        if let Some(span) = span {
            let ext = span.extensions();
            let data = ext
                .get::<FormattedFields<N>>()
                .expect("Unable to find FormattedFields in extensions; this is a bug");
            let fields = serde_json::from_str::<serde_json::Value>(data).map_err(|_| Error)?;
            if let Value::Object(map) = fields {
                attributes.extend(map);
            }
        }

        // Write JSON
        (|| {
            let mut serializer = serde_json::Serializer::new(WriteAdaptor::new(&mut writer));
            let mut log_map = serializer.serialize_map(None)?;
            log_map.serialize_entry("Timestamp", &format_args!("{}", timestamp))?;
            if let Some(trace_id) = trace_id {
                log_map.serialize_entry("TraceId", &format_args!("{:032x}", trace_id))?;
            }
            if let Some(span_id) = span_id {
                log_map.serialize_entry("SpanId", &format_args!("{:016x}", span_id))?;
            }
            log_map.serialize_entry("severity", severity_text)?;
            log_map.serialize_entry("SeverityText", severity_text)?;
            log_map.serialize_entry("SeverityNumber", &severity_number)?;
            log_map.serialize_entry("Body", &body)?;
            log_map.serialize_entry("Attributes", &attributes)?;
            log_map.end()
        })()
        .map_err(|_| std::fmt::Error)?;

        writeln!(writer)
    }
}
