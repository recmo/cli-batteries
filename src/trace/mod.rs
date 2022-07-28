#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

mod json_format;
mod open_telemetry;
mod span_formatter;
mod tiny_log_fmt;
mod tokio_console;

use self::{json_format::JsonFormatter, span_formatter::SpanFormatter, tiny_log_fmt::TinyLogFmt};
use crate::{default_from_clap, Version};
use clap::Parser;
use core::str::FromStr;
use eyre::{bail, eyre, Error as EyreError, Result as EyreResult, WrapErr as _};
use once_cell::sync::OnceCell;
use std::{
    cmp::max, env, fs::File, io::BufWriter, path::PathBuf, process::id as pid,
    thread::available_parallelism,
};
use tracing::{info, Level, Subscriber};
use tracing_error::ErrorLayer;
use tracing_flame::{FlameLayer, FlushGuard};
use tracing_log::{InterestCacheConfig, LogTracer};
use tracing_subscriber::{
    filter::Targets,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Layer, Registry,
};
use users::{get_current_gid, get_current_uid};

static FLAME_FLUSH_GUARD: OnceCell<Option<FlushGuard<BufWriter<File>>>> = OnceCell::new();

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Hash, Eq)]
enum LogFormat {
    Tiny,
    Compact,
    Pretty,
    Json,
}

impl LogFormat {
    fn into_layer<S>(self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
    {
        let layer = fmt::Layer::new()
            .with_writer(std::io::stderr)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
        match self {
            Self::Tiny => Box::new(
                layer
                    .event_format(TinyLogFmt::default())
                    .fmt_fields(TinyLogFmt::default())
                    .map_event_format(SpanFormatter::new),
            ) as Box<dyn Layer<S> + Send + Sync>,
            Self::Compact => Box::new(layer.compact().map_event_format(SpanFormatter::new)),
            Self::Pretty => Box::new(layer.pretty().map_event_format(SpanFormatter::new)),
            Self::Json => Box::new(
                layer
                    .json()
                    .with_current_span(true)
                    .with_span_list(false)
                    .event_format(JsonFormatter)
                    .map_event_format(SpanFormatter::new),
            ),
        }
    }
}

impl FromStr for LogFormat {
    type Err = EyreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "tiny" => Self::Tiny,
            "compact" => Self::Compact,
            "pretty" => Self::Pretty,
            "json" => Self::Json,
            _ => bail!("Invalid log format: {}", s),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, env, parse(from_occurrences))]
    verbose: usize,

    /// Apply an env_filter compatible log filter
    #[clap(long, env, default_value_t)]
    log_filter: String,

    /// Log format, one of 'tiny', 'compact', 'pretty' or 'json'
    #[clap(long, env, default_value = "tiny")]
    log_format: LogFormat,

    /// Store traces in a flame graph file for processing with inferno.
    #[clap(long, env)]
    trace_flame: Option<PathBuf>,

    #[cfg(feature = "tokio-console")]
    #[clap(flatten)]
    pub tokio_console: tokio_console::Options,

    #[cfg(feature = "otlp")]
    #[clap(flatten)]
    open_telemetry: open_telemetry::Options,
}

default_from_clap!(Options);

impl Options {
    #[allow(clippy::borrow_as_ptr)] // ptr::addr_of! does not work here.
    pub fn init(&self, version: &Version, load_addr: usize) -> EyreResult<()> {
        // Hack: ENV parsing for a `parse(from_occurrences)` argument
        // is not supported. So we have to do it manually.
        let verbose = env::var("VERBOSE")
            .ok()
            .and_then(|s| s.parse().ok())
            .map_or(self.verbose, |e| max(e, self.verbose));

        // Log filtering is a combination of `--log-filter` and `--verbose` arguments.
        let verbosity = {
            let (all, app) = match verbose {
                0 => (Level::ERROR, Level::INFO),
                1 => (Level::INFO, Level::INFO),
                2 => (Level::INFO, Level::DEBUG),
                3 => (Level::INFO, Level::TRACE),
                4 => (Level::DEBUG, Level::TRACE),
                _ => (Level::TRACE, Level::TRACE),
            };
            Targets::new()
                .with_default(all)
                .with_targets(version.app_crates.iter().map(|c| (c, app)))
        };
        let log_filter = if self.log_filter.is_empty() {
            Targets::new()
        } else {
            self.log_filter
                .parse()
                .wrap_err("Error parsing log-filter")?
        };
        let targets = verbosity.with_targets(log_filter);

        // Tracing stack
        let subscriber = Registry::default();

        // OpenTelemetry layer
        #[cfg(feature = "otlp")]
        let subscriber = subscriber.with(
            self.open_telemetry
                .to_layer(version)?
                .with_filter(targets.clone()),
        );

        // Optional trace flame layer
        let (flame, guard) = match self
            .trace_flame
            .as_ref()
            .map(FlameLayer::with_file)
            .transpose()?
        {
            Some((flame, guard)) => (Some(flame), Some(guard)),
            None => (None, None),
        };
        let subscriber = subscriber.with(flame);
        FLAME_FLUSH_GUARD
            .set(guard)
            .map_err(|_| eyre!("flame flush guard already initialized"))?;

        // Tokio Console layer
        #[cfg(feature = "tokio-console")]
        let subscriber = subscriber.with(self.tokio_console.into_layer());

        // Include span traces in errors
        let subscriber = subscriber.with(ErrorLayer::default());

        // Log output
        let subscriber = subscriber.with(self.log_format.into_layer().with_filter(targets));

        // Install
        tracing::subscriber::set_global_default(subscriber)?;

        // Route `log` crate events to `tracing`
        LogTracer::builder()
            .with_interest_cache(InterestCacheConfig::default())
            .init()?;

        // Log version information
        info!(
            host = version.target,
            pid = pid(),
            uid = get_current_uid(),
            gid = get_current_gid(),
            cores = available_parallelism()?,
            main = load_addr,
            commit = &version.commit_hash[..8],
            "{name} {version}",
            name = version.crate_name,
            version = version.pkg_version,
        );

        Ok(())
    }
}

pub fn shutdown() -> EyreResult<()> {
    if let Some(Some(flush_guard)) = FLAME_FLUSH_GUARD.get() {
        flush_guard.flush()?;
    }

    #[cfg(feature = "otlp")]
    open_telemetry::shutdown();

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_parse_args() {
        let cmd = "arg0 -v --log-filter foo -vvv";
        let options = Options::from_iter_safe(cmd.split(' ')).unwrap();
        assert_eq!(options, Options {
            verbose: 4,
            log_filter: "foo".to_owned(),
            log_format: LogFormat::Tiny,
            trace_flame: None,
            #[cfg(feature = "tokio-console")]
            tokio_console: tokio_console::Options::default(),
            #[cfg(feature = "otlp")]
            open_telemetry: open_telemetry::Options::default(),
        });
    }
}
