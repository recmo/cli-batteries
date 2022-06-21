#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use crate::{default_from_structopt, Version};
use core::str::FromStr;
use eyre::{bail, Error as EyreError, Result as EyreResult, WrapErr as _};
use std::{process::id as pid, thread::available_parallelism};
use structopt::StructOpt;
use tracing::{info, Level, Subscriber};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::Targets,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Layer, Registry,
};
use users::{get_current_gid, get_current_uid};

#[cfg(feature = "opentelemetry")]
use crate::open_telemetry;

#[cfg(feature = "tokio-console")]
use crate::tokio_console;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Hash, Eq)]
enum LogFormat {
    Compact,
    Pretty,
    Json,
}

impl LogFormat {
    fn into_layer<S>(self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
    {
        let layer = fmt::Layer::new().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
        match self {
            Self::Compact => Box::new(layer.compact()) as Box<dyn Layer<S> + Send + Sync>,
            Self::Pretty => Box::new(layer.pretty()),
            Self::Json => Box::new(layer.json()),
        }
    }
}

impl FromStr for LogFormat {
    type Err = EyreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "compact" => Self::Compact,
            "pretty" => Self::Pretty,
            "json" => Self::Json,
            _ => bail!("Invalid log format: {}", s),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Apply an env_filter compatible log filter
    #[structopt(long, env, default_value)]
    log_filter: String,

    /// Log format, one of 'compact', 'pretty' or 'json'
    #[structopt(long, env, default_value = "pretty")]
    log_format: LogFormat,

    #[cfg(feature = "tokio-console")]
    #[structopt(flatten)]
    pub tokio_console: tokio_console::Options,

    #[cfg(feature = "opentelemetry")]
    #[structopt(flatten)]
    open_telemetry: open_telemetry::Options,
}

default_from_structopt!(Options);

impl Options {
    #[allow(clippy::borrow_as_ptr)] // ptr::addr_of! does not work here.
    pub fn init(&self, version: &Version, load_addr: usize) -> EyreResult<()> {
        // Log filtering is a combination of `--log-filter` and `--verbose` arguments.
        let verbosity = {
            let (all, app) = match self.verbose {
                0 => (Level::ERROR, Level::INFO),
                1 => (Level::INFO, Level::INFO),
                2 => (Level::INFO, Level::DEBUG),
                3 => (Level::INFO, Level::TRACE),
                4 => (Level::DEBUG, Level::TRACE),
                _ => (Level::TRACE, Level::TRACE),
            };
            Targets::new()
                .with_default(all)
                .with_target(version.pkg_name.replace('-', "_"), app)
                .with_target(version.crate_name.replace('-', "_"), app)
        };
        let log_filter = if self.log_filter.is_empty() {
            Targets::new()
        } else {
            self.log_filter
                .parse()
                .wrap_err("Error parsing log-filter")?
        };
        let targets = verbosity.with_targets(log_filter);

        // Route events to both tokio-console and stdout
        let subscriber = Registry::default();

        #[cfg(feature = "tokio-console")]
        let subscriber = subscriber.with(self.tokio_console.into_layer());

        #[cfg(feature = "opentelemetry")]
        let subscriber = subscriber.with(self.open_telemetry.to_layer()?);

        let subscriber = subscriber
            .with(ErrorLayer::default())
            .with(self.log_format.into_layer().with_filter(targets));
        tracing::subscriber::set_global_default(subscriber)?;

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

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_parse_args() {
        let cmd = "arg0 -v --log-filter foo -vvv";
        let options = Options::from_iter_safe(cmd.split(' ')).unwrap();
        assert_eq!(options, Options {
            verbose:        4,
            log_filter:     "foo".to_owned(),
            log_format:     LogFormat::Pretty,
            tokio_console:  tokio_console::Options::default(),
            open_telemetry: open_telemetry::Options::default(),
        });
    }
}
