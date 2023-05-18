use opentelemetry::trace::TraceContextExt;
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
    let span_ref = span_from_ctx(ctx)?;

    let extensions = span_ref.extensions();

    let data = extensions.get::<OtelData>()?;
    let parent_trace_id = data.parent_cx.span().span_context().trace_id();
    let parent_trace_id_u128 = u128::from_be_bytes(parent_trace_id.to_bytes());

    // So parent trace id will usually be zero UNLESS we extract a trace id from
    // headers in which case it'll be the trace id from headers. And for some
    // reason this logic is not handled with Option
    //
    // So in case the parent trace id is zero, we should use the builder trace id.
    if parent_trace_id_u128 == 0 {
        let builder_id = data.builder.trace_id?;

        Some(u128::from_be_bytes(builder_id.to_bytes()))
    } else {
        Some(parent_trace_id_u128)
    }
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
    let span_ref = span_from_ctx(ctx)?;

    let extensions = span_ref.extensions();

    let data = extensions.get::<OtelData>()?;
    let parent_span_id = data.parent_cx.span().span_context().span_id();
    let parent_span_id_u64 = u64::from_be_bytes(parent_span_id.to_bytes());

    // Same logic as for trace ids
    if parent_span_id_u64 == 0 {
        let builder_id = data.builder.span_id?;

        Some(u64::from_be_bytes(builder_id.to_bytes()))
    } else {
        Some(parent_span_id_u64)
    }
}

fn span_from_ctx<'a, S, N>(ctx: &'a FmtContext<'a, S, N>) -> Option<SpanRef<'a, S>>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span = ctx.lookup_current().or_else(|| ctx.parent_span());

    span
}
