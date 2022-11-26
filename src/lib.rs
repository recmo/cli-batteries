// TODO:
// https://crates.io/crates/shadow-rs
// https://crates.io/crates/argfile
// https://docs.rs/wild/latest/wild/
// https://crates.io/crates/clap_complete

#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

mod allocator;
mod build;
mod heartbeat;
mod metered_allocator;
mod prometheus;
mod rand;
mod rayon;
mod shutdown;
mod trace;
mod version;

pub use crate::{
    build::build_rs,
    heartbeat::heartbeat,
    shutdown::{await_shutdown, is_shutting_down, shutdown},
    version::Version,
};
use clap::{Args, CommandFactory, FromArgMatches, Parser};
use eyre::{Error as EyreError, Report, Result as EyreResult, WrapErr};
use std::{future::Future, ptr::addr_of};
use tokio::runtime;
use tracing::{error, info};

#[cfg(feature = "mock-shutdown")]
pub use crate::shutdown::reset_shutdown;

#[cfg(feature = "metered-allocator")]
use crate::metered_allocator::MeteredAllocator;

#[cfg(feature = "otlp")]
pub use crate::trace::{trace_from_headers, trace_to_headers};

/// Implement [`Default`] for a type that implements [`Parser`] and has
/// default values set for all fields.
#[macro_export]
macro_rules! default_from_clap {
    ($ty:ty) => {
        impl ::std::default::Default for $ty {
            fn default() -> Self {
                use ::clap::Parser;
                use ::std::ffi::OsString;
                <Self as Parser>::parse_from::<Option<OsString>, OsString>(None)
            }
        }
    };
}

// TODO: Use the new command / arg distinction from clap.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Parser)]
#[group(skip)]
struct Options<O: Args> {
    #[clap(flatten)]
    tracing: trace::Options,

    #[cfg(feature = "rand")]
    #[clap(flatten)]
    rand: rand::Options,

    #[cfg(feature = "rayon")]
    #[clap(flatten)]
    rayon: rayon::Options,

    #[cfg(feature = "prometheus")]
    #[clap(flatten)]
    prometheus: prometheus::Options,

    #[clap(flatten)]
    app: O,
}

/// Run the program.
#[allow(clippy::needless_pass_by_value)]
pub fn run<A, O, F, E>(version: Version, app: A)
where
    A: FnOnce(O) -> F,
    O: Args,
    F: Future<Output = Result<(), E>>,
    E: Into<Report> + Send + Sync + 'static,
{
    if let Err(report) = run_fallible(&version, app) {
        error!(?report, "{}", report);
        error!("Program terminating abnormally");
        std::process::exit(1);
    }
}

fn run_fallible<A, O, F, E>(version: &Version, app: A) -> EyreResult<()>
where
    A: FnOnce(O) -> F,
    O: Args,
    F: Future<Output = Result<(), E>>,
    E: Into<Report> + Send + Sync + 'static,
{
    // Install panic handler
    // TODO: write panics to log, like Err results.
    color_eyre::config::HookBuilder::default()
        .issue_url(format!("{}/issues/new", version.pkg_repo))
        .add_issue_metadata(
            "version",
            format!("{} {}", version.pkg_name, version.long_version),
        )
        .install()
        .map_err(|err| {
            eprintln!("Error: {}", err);
            err
        })?;

    // Parse CLI and handle help and version (which will stop the application).
    let matches = Options::<O>::command()
        .name(version.pkg_name)
        .version(version.pkg_version)
        .long_version(version.long_version)
        .get_matches();
    let options = Options::<O>::from_arg_matches(&matches)?;

    // Start allocator metering (if enabled)
    allocator::start_metering();

    // TODO: Early logging to catch errors before we start the runtime.

    // Launch Tokio runtime
    // TODO: https://docs.rs/tokio/latest/tokio/runtime/struct.Builder.html#method.unhandled_panic
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .wrap_err("Error creating Tokio runtime")?
        .block_on(async {
            // Start heartbeat
            let heartbeat = tokio::spawn(heartbeat());

            // Monitor for Ctrl-C
            #[cfg(feature = "signals")]
            shutdown::watch_signals();

            // Start log system
            let load_addr = addr_of!(app) as usize;
            options.tracing.init(version, load_addr).map_err(|err| {
                eprintln!("Error: {}", err);
                err
            })?;

            #[cfg(feature = "rand")]
            options.rand.init();

            #[cfg(feature = "rayon")]
            options.rayon.init()?;

            // Start prometheus
            #[cfg(feature = "prometheus")]
            let prometheus = tokio::spawn(prometheus::main(options.prometheus));

            // Start main
            app(options.app).await.map_err(E::into)?;

            // Initiate shutdown if main returns
            shutdown::shutdown();

            // Wait for prometheus to finish
            #[cfg(feature = "prometheus")]
            prometheus.await??;

            // Submit remaining traces
            trace::shutdown()?;

            // Join heartbeat thread
            heartbeat.await?;

            Result::<(), EyreError>::Ok(())
        })?;

    // Terminate successfully
    info!("Program terminating normally");
    Ok(())
}

#[cfg(test)]
pub mod test {
    use tracing::{error, info, warn};
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_with_log_output() {
        error!("logged on the error level");
        assert!(logs_contain("logged on the error level"));
    }

    #[tokio::test]
    #[traced_test]
    #[allow(clippy::semicolon_if_nothing_returned)] // False positive
    async fn async_test_with_log() {
        // Local log
        info!("This is being logged on the info level");

        // Log from a spawned task (which runs in a separate thread)
        tokio::spawn(async {
            warn!("This is being logged on the warn level from a spawned task");
        })
        .await
        .unwrap();

        // Ensure that `logs_contain` works as intended
        assert!(logs_contain("logged on the info level"));
        assert!(logs_contain("logged on the warn level"));
        assert!(!logs_contain("logged on the error level"));
    }
}
