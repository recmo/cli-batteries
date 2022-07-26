use ansi_term::{Colour, Style};
use std::{
    fmt::{Debug, Error, Result, Write},
    time::Instant,
};
use tracing::{
    field::{Field, Visit},
    span::Record,
    Event, Level, Subscriber,
};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    field::{MakeVisitor, RecordFields, VisitFmt, VisitOutput},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::{LookupSpan, Scope},
};

pub struct TinyLogFmt {
    epoch: Instant,
}

struct TinyFields;

struct TinyVisitor<'a> {
    writer:   Writer<'a>,
    is_empty: bool,
    result:   Result,
}

impl Default for TinyLogFmt {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for TinyLogFmt
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

        // Fields
        ctx.format_fields(writer.by_ref(), event)?;

        // Span attributes
        if meta.is_span() {
            let span = event
                .parent()
                .and_then(|id| ctx.span(id))
                .ok_or_else(Error::default)?;

            let exts = span.extensions();
            if let Some(fields) = exts.get::<FormattedFields<N>>() {
                if !fields.is_empty() {
                    write!(writer, " {}", &fields.fields)?;
                }
            }
        }

        writeln!(writer)?;
        Ok(())
    }
}

impl<'writer> FormatFields<'writer> for TinyLogFmt {
    fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> Result {
        let mut v = TinyVisitor::new(writer, true);
        fields.record(&mut v);
        v.finish()
    }

    fn add_fields(
        &self,
        current: &'writer mut FormattedFields<Self>,
        fields: &Record<'_>,
    ) -> Result {
        let empty = current.is_empty();
        let writer = current.as_writer();
        let mut v = TinyVisitor::new(writer, empty);
        fields.record(&mut v);
        v.finish()
    }
}

impl<'a> MakeVisitor<Writer<'a>> for TinyFields {
    type Visitor = TinyVisitor<'a>;

    fn make_visitor(&self, mut writer: Writer<'a>) -> Self::Visitor {
        TinyVisitor::new(writer, true)
    }
}

impl<'a> TinyVisitor<'a> {
    const fn new(writer: Writer<'a>, is_empty: bool) -> Self {
        Self {
            writer,
            is_empty,
            result: Ok(()),
        }
    }

    fn write_padded(&mut self, value: &impl Debug) {
        let padding = if self.is_empty {
            self.is_empty = false;
            ""
        } else {
            " "
        };
        self.result = write!(self.writer, "{}{:?}", padding, value);
    }
}

impl<'a> Visit for TinyVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_err() {
            return;
        }
        if field.name() == "message" {
            self.record_debug(field, &format_args!("{}", value))
        } else {
            self.record_debug(field, &value)
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_debug(field, &format_args!("{}", value))
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if self.result.is_err() {
            return;
        }

        let message_style = Style::default();
        let trace_style = Style::default().italic();
        let key_style = Style::default().dimmed().italic();
        let value_style = Style::default();

        match field.name() {
            "message" => self.write_padded(&format_args!(
                "{}{:?}{}",
                message_style.prefix(),
                value,
                message_style.suffix()
            )),
            "span" => self.write_padded(&format_args!(
                "({}{:?}{})",
                trace_style.prefix(),
                value,
                trace_style.suffix()
            )),
            // Skip fields that are actually log metadata that have already been handled
            name if name.starts_with("log.") => {}
            name if name.starts_with("r#") => self.write_padded(&format_args!(
                "{}{}:{}{:?}",
                key_style.prefix(),
                &name[2..],
                key_style.infix(value_style),
                value
            )),
            name => self.write_padded(&format_args!(
                "{}{}:{}{:?}",
                key_style.prefix(),
                name,
                key_style.infix(value_style),
                value
            )),
        };
    }
}

impl<'a> VisitOutput<Result> for TinyVisitor<'a> {
    fn finish(mut self) -> Result {
        let style = Style::default();
        write!(&mut self.writer, "{}", style.suffix())?;
        self.result
    }
}

impl<'a> VisitFmt for TinyVisitor<'a> {
    fn writer(&mut self) -> &mut dyn Write {
        &mut self.writer
    }
}
