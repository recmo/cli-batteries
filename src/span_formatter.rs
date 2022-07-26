use ansi_term::{Colour, Style};
use itertools::Itertools;
use std::{
    fmt::{Debug, Error, Result},
    marker::PhantomData,
    time::Instant,
};
use tracing::{
    field::{display, Field, FieldSet, Visit},
    Event, Level, Subscriber, Value,
};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    field::RecordFields,
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

pub struct SpanFormatter<Inner, S, N>
where
    Inner: FormatEvent<S, N>,
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    inner:    Inner,
    _phantom: PhantomData<(S, N)>,
}

impl<'b, Inner, S, N> SpanFormatter<Inner, S, N>
where
    Inner: FormatEvent<S, N>,
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'b, Inner, S, N> FormatEvent<S, N> for SpanFormatter<Inner, S, N>
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
        let meta = event.metadata();
        if meta.is_span() {
            // Extract fields from event
            #[derive(Debug, Default)]
            struct Visitor {
                time_busy: Option<String>,
                time_idle: Option<String>,
            };
            impl Visit for Visitor {
                fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                    match field.name() {
                        "time.busy" => self.time_busy = Some(format!("{:?}", value)),
                        "time.idle" => self.time_idle = Some(format!("{:?}", value)),
                        _ => (),
                    }
                }
            }
            let mut visitor = Visitor::default();
            event.record(&mut visitor);

            // Get fields
            let callsite = meta.callsite();
            let span = event
                .parent()
                .and_then(|id| ctx.span(id))
                .ok_or(Error::default())?;
            let message = span.name();
            if let (Some(time_busy), Some(time_idle)) = (visitor.time_busy, visitor.time_idle) {
                // Closing event
                let time_busy = display(time_busy);
                let time_idle = display(time_idle);
                let span = display("end");
                let field_set =
                    FieldSet::new(&["message", "span", "time.busy", "time.idle"], callsite);
                let mut fields = field_set.iter();
                let values = [
                    (&fields.next().unwrap(), Some(&message as &dyn Value)),
                    (&fields.next().unwrap(), Some(&span)),
                    (&fields.next().unwrap(), Some(&time_busy)),
                    (&fields.next().unwrap(), Some(&time_idle)),
                ];
                let value_set = field_set.value_set(&values);
                let event =
                    Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);
                self.inner.format_event(ctx, writer, &event)?;
            } else {
                // Opening event
                let span = display("begin");
                let field_set = FieldSet::new(&["message", "span"], callsite);
                let mut fields = field_set.iter();
                let values = [
                    (&fields.next().unwrap(), Some(&message as &dyn Value)),
                    (&fields.next().unwrap(), Some(&span)),
                ];
                let value_set = field_set.value_set(&values);
                let event =
                    Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);
                self.inner.format_event(ctx, writer, &event)?;
            }
        } else {
            self.inner.format_event(ctx, writer, &event)?;
        }
        Ok(())
    }
}
