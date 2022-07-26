use ansi_term::{Colour, Style};
use itertools::Itertools;
use std::{
    fmt::{Debug, Result, Error},
    time::Instant,
};
use tracing::{
    field::{Field, Visit, FieldSet, display},
    Event, Level, Subscriber,
};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    field::RecordFields,
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};
use tracing::Value;

pub struct LogFmt {
    epoch: Instant,
}

impl Default for LogFmt {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for LogFmt
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> Result {
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        let dimmed = Style::new().dimmed();
        let bold = Style::new().bold();

        // Uptime
        let e = self.epoch.elapsed();
        write!(writer, "{}", dimmed.prefix())?;
        write!(writer, "{:4}.{:06} ", e.as_secs(), e.subsec_micros())?;
        write!(writer, "{}", dimmed.suffix())?;

        // Log level
        write!(writer, "{}", bold.prefix())?;
        write!(writer, "{} ", match *meta.level() {
            Level::TRACE => Colour::Purple.paint("T"),
            Level::DEBUG => Colour::Blue.paint("D"),
            Level::INFO => Colour::Green.paint("I"),
            Level::WARN => Colour::Yellow.paint("W"),
            Level::ERROR => Colour::Red.paint("E"),
        })?;
        write!(writer, "{}", bold.suffix())?;

        if meta.is_span() {
            // Extract fields from event
            #[derive(Debug,Default)]
            struct Visitor{
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
                let field_set = FieldSet::new(&[
                    "message",
                    "span",
                    "time.busy",
                    "time.idle",
                    ], callsite);
                let mut fields = field_set.iter();
                let values = [
                    (&fields.next().unwrap(), Some(&message as  &dyn Value)),
                    (&fields.next().unwrap(), Some(&span)),
                    (&fields.next().unwrap(), Some(&time_busy)),
                    (&fields.next().unwrap(), Some(&time_idle))
                ];
                let value_set = field_set.value_set(&values);
                let event = Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);
                ctx.format_fields(writer.by_ref(), event);
            } else {
                // Opening event
                let span = display("begin");
                let field_set = FieldSet::new(&["message", "span"], callsite);
                let mut fields = field_set.iter();
                let values = [
                    (&fields.next().unwrap(), Some(&message as  &dyn Value)),
                    (&fields.next().unwrap(), Some(&span))
                ];
                let value_set = field_set.value_set(&values);
                let event = Event::new_child_of(event.parent().cloned(), event.metadata(), &value_set);
                ctx.format_fields(writer.by_ref(), event);
            }

        } else {
            ctx.format_fields(writer.by_ref(), event);
        }

        writeln!(writer)?;

        Ok(())
    }
}
