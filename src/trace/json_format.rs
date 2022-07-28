use chrono::Utc;
use opentelemetry::trace::{SpanId, TraceContextExt};
use serde::{
    ser::{SerializeMap, Serializer as _},
    Serializer,
};
use serde_json::Value;
use std::{
    fmt::{Error, Result},
    io,
    marker::PhantomData,
};
use tracing::{span::Attributes, Event, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_serde::{fields::AsMap, AsSerde};
use tracing_subscriber::{
    fmt::{
        format::{JsonFields, Writer},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    registry::{LookupSpan, SpanRef},
};

// See https://github.com/tokio-rs/tracing/issues/1531#issuecomment-988172764

#[cfg(feature = "otlp")]
use opentelemetry::trace::SpanBuilder;

pub struct JsonFormatter;

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

impl<S, N> FormatEvent<S, N> for JsonFormatter
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

        let span = if meta.is_span() {
            event.parent().and_then(|id| ctx.span(id))
        } else {
            None
        };

        (||{
            let mut serializer = serde_json::Serializer::new(WriteAdaptor::new(&mut writer));

            let mut serializer = serializer.serialize_map(None)?;
            serializer.serialize_entry("timestamp", &Utc::now().to_rfc3339())?;
            serializer.serialize_entry("level", &meta.level().as_serde())?;
            serializer.serialize_entry("target", meta.target())?;

            if let Some(span) = span {
                // Merge field and span attributes
                let mut fields = serde_json::to_value(&event.field_map())?;
                let ext = span.extensions();
                let data = ext
                    .get::<FormattedFields<N>>()
                    .expect("Unable to find FormattedFields in extensions; this is a bug");
                let attributes = serde_json::from_str::<serde_json::Value>(data)?;
                if let (Value::Object(target), Value::Object(source)) = (&mut fields, attributes) {
                    target.extend(source.into_iter());
                }
                serializer.serialize_entry("fields", &fields)?;
            } else {
                serializer.serialize_entry("fields", &event.field_map())?;
            }

            #[cfg(feature = "otlp")]
            if let Some(mut span_ref) = ctx.lookup_current() {
                // Find span_id
                if let Some(builder) = span_ref.extensions().get::<SpanBuilder>() &&
                    let Some(span_id) = builder.span_id {
                    let span_id = format!("{}", span_id);
                    serializer.serialize_entry("span_id", &span_id)?;
                    serializer.serialize_entry("dd.span_id", &span_id)?;
                }
                // Find trace_id by going up the stack
                loop {
                    dbg!(span_ref.extensions().get::<SpanBuilder>());
                    dbg!(lookup_trace_info(&span_ref));
                    if let Some(builder) = span_ref.extensions().get::<SpanBuilder>() {
                        if let Some(trace_id) = builder.trace_id {
                            let trace_id = format!("{}", trace_id);
                            serializer.serialize_entry("trace_id", &trace_id)?;
                            let suffix = if trace_id.len() > 16 {
                                &trace_id[(trace_id.len() - 16)..]
                            } else {
                                &trace_id[..]
                            };
                            serializer.serialize_entry("dd.trace_id", suffix)?;
                            break;
                        }
                    }
                    if let Some(parent) = span_ref.parent() {
                        span_ref = parent;
                    } else {
                        break;
                    }
                }
            }

            serializer.end()
        })().map_err(|_| std::fmt::Error)?;

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
