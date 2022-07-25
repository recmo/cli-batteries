use ansi_term::{Colour, Style};
use itertools::Itertools;
use std::{
    fmt::{Debug, Result},
    time::Instant,
};
use tracing::{
    field::{Field, Visit},
    Event, Level, Subscriber,
};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    field::RecordFields,
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

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
            // Heuristic to detect open or close span
            let open = !event
                .fields()
                .find(|field| field.name() == "time.busy")
                .is_some();

            let name = event
                .parent()
                .and_then(|id| ctx.span(id))
                .map_or("unknown", |span| span.name());

            // Create event with augmented message
            let values = event.fields().map(|field| {
                match field.name() {
                    _ => 
                }
            }).collect::<Vec<_>>();
            let vs = fs.value_set(&v);
            let event = Event::new_child_of(event.parent(), meta, &vs);

        }

        // struct Visitor<'a>(Writer<'a>);
        // impl<'a> Visit for Visitor<'a> {
        //     fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        //         write!(self.0, "{} = {:?}", field, value);
        //     }
        // }
        // let mut visitor = Visitor(writer);
        // event.record(&mut visitor);
        // let mut writer = visitor.0;

        ctx.format_fields(writer.by_ref(), event);

        writeln!(writer)?;

        Ok(())
    }
}
