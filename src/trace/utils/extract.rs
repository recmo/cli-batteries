use tracing::Subscriber;
use tracing_opentelemetry::OtelData;
use tracing_subscriber::{
    fmt::{FmtContext, FormatFields},
    registry::{LookupSpan, SpanRef},
};

/// Finds Otel trace id by going up the span stack until we find a span
/// with a trace id.
pub fn opentelemetry_trace_id<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u128>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let mut scope = ctx.event_scope()?;

    scope.find_map(|span| {
        let extensions = span.extensions();

        let otel_data = extensions.get::<OtelData>()?;

        let id = otel_data.builder.trace_id?;

        Some(u128::from_be_bytes(id.to_bytes()))
    })
}

/// Finds Otel span id
///
/// BUG: The otel object is not available for span end events. This is
/// because the Otel layer is higher in the stack and removes the
/// extension before we get here.
///
/// Fallbacks on tracing span id
pub fn opentelemetry_span_id<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u64>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let otel_span_id = opentelemetry_span_id_inner(ctx);
    let tracing_span_id = tracing_span_id(ctx);

    otel_span_id.or(tracing_span_id)
}

fn tracing_span_id<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u64>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span = span_from_ctx(ctx)?;

    Some(span.id().into_u64())
}

fn opentelemetry_span_id_inner<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u64>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span = span_from_ctx(ctx)?;

    let extensions = span.extensions();

    let otel = extensions.get::<OtelData>()?;

    let id = otel.builder.span_id?;

    Some(u64::from_be_bytes(id.to_bytes()))
}

fn span_from_ctx<'a, S, N>(ctx: &'a FmtContext<'a, S, N>) -> Option<SpanRef<'a, S>>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span = ctx.parent_span().or_else(|| ctx.lookup_current());

    span
}
