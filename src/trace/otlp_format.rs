use chrono::Utc;
use opentelemetry::trace::{SpanId, TraceContextExt};
use serde::{
    ser::{SerializeMap, Serializer as _},
    Serializer,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::{Error, Result},
    io,
    marker::PhantomData,
    thread,
};
use tracing::{span::Attributes, Event, Level, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_serde::{fields::AsMap, AsSerde};
use tracing_subscriber::{
    fmt::{
        format::{JsonFields, Writer},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    registry::{LookupSpan, SpanRef},
};

// Implements <https://opentelemetry.io/docs/reference/specification/logs/data-model/>
// <https://opentelemetry.io/docs/reference/specification/logs/semantic_conventions/>
// <https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/>

// See https://github.com/tokio-rs/tracing/issues/1531#issuecomment-988172764

#[cfg(feature = "otlp")]
use opentelemetry::trace::SpanBuilder;

pub struct OtlpFormatter;

#[derive(Debug)]
pub struct TraceInfo {
    pub trace_id: String,
    pub span_id:  String,
}

fn lookup_trace_info<S>(span_ref: &SpanRef<S>) -> Option<TraceInfo>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    dbg!(span_ref.extensions());
    dbg!(span_ref.extensions().get::<OtelData>()).map(|o| TraceInfo {
        trace_id: o.parent_cx.span().span_context().trace_id().to_string(),
        span_id:  o.builder.span_id.unwrap_or(SpanId::INVALID).to_string(),
    })
}

impl<S, N> FormatEvent<S, N> for OtlpFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
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
        let timestamp = Utc::now().timestamp_nanos();
        let trace_id = Some(0_u64); // TODO
        let span_id = Some(0_u64); // TODO
        let (severity_text, severity_number) = match *meta.level() {
            Level::TRACE => ("TRACE", 1),
            Level::DEBUG => ("DEBUG", 5),
            Level::INFO => ("INFO", 9),
            Level::WARN => ("WARN", 13),
            Level::ERROR => ("ERROR", 17),
        };
        let mut body = String::new();
        let mut attributes = serde_json::Map::<String, Value>::new();

        // Find span and trace id
        // if let Some(mut span_ref) = ctx.lookup_current() {
        //     // Find span_id
        //     if let Some(builder) = span_ref.extensions().get::<SpanBuilder>() &&
        //             let Some(span_id) = builder.span_id {
        //             let span_id = format!("{}", span_id);
        //             serializer.serialize_entry("span_id", &span_id)?;
        //             serializer.serialize_entry("dd.span_id", &span_id)?;
        //         }
        //     // Find trace_id by going up the stack
        //     loop {
        //         dbg!(span_ref.extensions().get::<SpanBuilder>());
        //         dbg!(lookup_trace_info(&span_ref));
        //         if let Some(builder) = span_ref.extensions().get::<SpanBuilder>() {
        //             if let Some(trace_id) = builder.trace_id {
        //                 let trace_id = format!("{}", trace_id);
        //                 serializer.serialize_entry("trace_id", &trace_id)?;
        //                 let suffix = if trace_id.len() > 16 {
        //                     &trace_id[(trace_id.len() - 16)..]
        //                 } else {
        //                     &trace_id[..]
        //                 };
        //                 serializer.serialize_entry("dd.trace_id", suffix)?;
        //                 break;
        //             }
        //         }
        //         if let Some(parent) = span_ref.parent() {
        //             span_ref = parent;
        //         } else {
        //             break;
        //         }
        //     }
        // }

        // https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/span-general/#source-code-attributes
        // attributes.insert("code.function".into(), meta.target().into());
        meta.module_path()
            .map(|s| attributes.insert("code.namespace".into(), s.into()));
        if let Some(filepath) = meta.file() {
            attributes.insert("code.filepath".into(), filepath.into());
        }
        if let Some(lineno) = meta.line() {
            // TODO: Encode as int?
            attributes.insert("code.lineno".into(), format!("{}", lineno).into());
        }

        // https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/span-general/#source-code-attributes
        // tracing-subscriber does. TODO (blocked): https://github.com/rust-lang/rust/issues/67939
        // attributes.insert("thread.id".into(), thread::current().id().as_u64());
        attributes.insert("thread.name".into(), match thread::current().name() {
            Some(name) => name.into(),
            None => format!("{:?}", thread::current().id()).into(),
        });

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
                    _ => Some((k,v))
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
                log_map.serialize_entry("TraceId", &format_args!("{:016x}", trace_id))?;
            }
            if let Some(span_id) = span_id {
                log_map.serialize_entry("SpanId", &format_args!("{:016x}", span_id))?;
            }
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

struct WriteAdaptor<'a> {
    fmt_write: &'a mut dyn std::fmt::Write,
}

impl<'a> WriteAdaptor<'a> {
    pub fn new(fmt_write: &'a mut dyn std::fmt::Write) -> Self {
        Self { fmt_write }
    }
}

impl<'a> io::Write for WriteAdaptor<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.fmt_write
            .write_str(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(s.as_bytes().len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
