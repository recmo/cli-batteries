use std::{fmt::Result, marker::PhantomData};

use tracing::{field::FieldSet, Event, Subscriber, Value};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

// See https://github.com/tokio-rs/tracing/issues/1531

#[allow(unused)]
pub enum EnrichmentMode {
    None,
    DataDog,
    OpenTelemetry,
}

pub struct EventEnrichmentCenter<Inner, S, N>
where
    Inner: FormatEvent<S, N>,
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    inner:           Inner,
    enrichment_mode: EnrichmentMode,
    _phantom:        PhantomData<(S, N)>,
}

impl<Inner, S, N> EventEnrichmentCenter<Inner, S, N>
where
    Inner: FormatEvent<S, N>,
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    pub const fn new(inner: Inner) -> Self {
        Self {
            inner,
            enrichment_mode: EnrichmentMode::DataDog,
            _phantom: PhantomData,
        }
    }

    pub fn _with_mode(mut self, mode: EnrichmentMode) -> Self {
        self.enrichment_mode = mode;
        self
    }
}

impl<Inner, S, N> FormatEvent<S, N> for EventEnrichmentCenter<Inner, S, N>
where
    Inner: FormatEvent<S, N>,
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: Writer<'_>,
        event: &Event<'_>,
    ) -> Result {
        if self.enrichment_mode.is_none() {
            return self.inner.format_event(ctx, writer, event);
        }

        let meta = event.metadata();

        if meta.is_event() {
            // ctx.current_span()
            // let span = Span::current().context();

            let (trace_id, span_id) = {
                let span = event
                    .parent()
                    .and_then(|id| ctx.span(id))
                    .or_else(|| ctx.lookup_current());

                let mut trace_id = None;
                let mut span_id = span.as_ref().map(|s| s.id().into_u64());

                // Find Otel span id
                // BUG: The otel object is not available for span end events. This is
                // because the Otel layer is higher in the stack and removes the
                // extension before we get here.
                span_id = span
                    .and_then(|span| {
                        let extensions = span.extensions();
                        extensions
                            .get::<OtelData>()
                            .and_then(|otel| otel.builder.span_id)
                            .map(|id| u64::from_be_bytes(id.to_bytes()))
                    })
                    .or(span_id); // Fallback to tracing span id

                // Find Otel trace id by going up the span stack until we find a span
                // with a trace id.
                trace_id = ctx
                    .event_scope()
                    .and_then(|mut scope| {
                        scope.find_map(|span| {
                            let extensions = span.extensions();
                            extensions
                                .get::<OtelData>()
                                .and_then(|otel| otel.builder.trace_id)
                                .map(|id| u128::from_be_bytes(id.to_bytes()))
                        })
                    })
                    .or(trace_id);

                (trace_id, span_id)
            };

            let callsite = meta.callsite();

            match self.enrichment_mode {
                EnrichmentMode::DataDog => {
                    let field_set = FieldSet::new(&["dd.trace_id", "dd.span_id"], callsite);

                    let mut fields = field_set.iter();
                    let values = [
                        (&fields.next().unwrap(), Some(&trace_id as &dyn Value)),
                        (&fields.next().unwrap(), Some(&span_id as &dyn Value)),
                    ];

                    let value_set = field_set.value_set(&values);

                    let event =
                        Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);

                    self.inner.format_event(ctx, writer, &event)
                }
                EnrichmentMode::OpenTelemetry => {
                    let field_set = FieldSet::new(&["TraceId", "SpanId"], callsite);

                    let mut fields = field_set.iter();
                    let values = [
                        (&fields.next().unwrap(), Some(&trace_id as &dyn Value)),
                        (&fields.next().unwrap(), Some(&span_id as &dyn Value)),
                    ];

                    let value_set = field_set.value_set(&values);

                    let event =
                        Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);

                    self.inner.format_event(ctx, writer, &event)
                }
                EnrichmentMode::None => unreachable!(),
            }
        } else {
            self.inner.format_event(ctx, writer, &event)
        }
    }
}

impl EnrichmentMode {
    pub fn is_none(&self) -> bool {
        matches!(self, EnrichmentMode::None)
    }
}
