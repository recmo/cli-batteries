#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

mod build;
mod logging;
mod prometheus;
mod rand;
mod rayon;
mod shutdown;
mod tokio_console;
mod version;

pub use crate::{build::build_rs, version::Version};
use eyre::{Error as EyreError, Result as EyreResult, WrapErr};
use std::{error::Error, future::Future, ptr::addr_of};
use structopt::StructOptInternal;
pub use structopt::{self, StructOpt};
use tokio::runtime;
use tracing::{error, info};

#[derive(StructOpt)]
struct Options<O: StructOpt + StructOptInternal> {
    #[structopt(flatten)]
    log: logging::Options,

    #[cfg(feature = "rand")]
    #[structopt(flatten)]
    rand: rand::Options,

    #[cfg(feature = "rayon")]
    #[structopt(flatten)]
    rayon: rayon::Options,

    #[cfg(feature = "prometheus")]
    #[structopt(flatten)]
    prometheus: prometheus::Options,

    #[structopt(flatten)]
    app: O,
}

pub fn run<A, O, F, E>(version: Version, app: A)
where
    A: FnOnce(O) -> F,
    O: StructOpt + StructOptInternal,
    F: Future<Output = Result<(), E>>,
    E: Error + Send + Sync + 'static,
{
    if let Err(report) = run_fallible(version, app) {
        error!(?report, "{}", report);
        error!("Program terminating abnormally");
        std::process::exit(1);
    }
}

fn run_fallible<A, O, F, E>(version: Version, app: A) -> EyreResult<()>
where
    A: FnOnce(O) -> F,
    O: StructOpt + StructOptInternal,
    F: Future<Output = Result<(), E>>,
    E: Error + Send + Sync + 'static,
{
    // Install panic handler
    // TODO: write panics to log, like Err results.
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", version.long_version)
        .install()?;

    // Parse CLI and handle help and version (which will stop the application).
    let matches = Options::<O>::clap()
        .name(version.pkg_name)
        .version(version.pkg_version)
        .long_version(version.long_version)
        .get_matches();
    let options = Options::<O>::from_clap(&matches);

    // Start log system
    let load_addr = addr_of!(app) as usize;
    options.log.init(&version, load_addr)?;

    #[cfg(feature = "rand")]
    options.rand.init();

    #[cfg(feature = "rayon")]
    options.rayon.init()?;

    // Launch Tokio runtime
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .wrap_err("Error creating Tokio runtime")?
        .block_on(async {
            // Monitor for Ctrl-C
            shutdown::watch_signals();

            // Start prometheus
            #[cfg(feature = "prometheus")]
            let prometheus = tokio::spawn(prometheus::main(options.prometheus));

            // Start main
            app(options.app).await?;

            // Initiate shutdown if main returns
            shutdown::shutdown();

            // Wait for prometheus to finish
            prometheus.await??;

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
