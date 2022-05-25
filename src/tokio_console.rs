use structopt::StructOpt;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

#[cfg(feature = "tokio-console")]
use console_subscriber::ConsoleLayer;

#[cfg(not(feature = "tokio-console"))]
use tracing_subscriber::layer::Identity;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StructOpt)]
pub struct Options {
    /// Start a tokio-console server on `http://127.0.0.1:6669/`.
    #[structopt(long)]
    #[cfg(feature = "tokio-console")]
    pub tokio_console: bool,
}

impl Options {
    #[cfg_attr(not(feature = "tokio-console"), allow(clippy::unused_self))]
    pub fn into_layer<S>(self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        #[cfg(not(feature = "tokio-console"))]
        return Identity::new();

        #[cfg(feature = "tokio-console")]
        if self.tokio_console {
            // TODO: Remove when <https://github.com/tokio-rs/tokio/issues/4114> resolves
            // TODO: Configure server addr.
            // TODO: Manage server thread.
            assert!(
                cfg!(tokio_unstable) && cfg!(feature = "tokio-console"),
                "Enabling --tokio-console requires a build with RUSTFLAGS=\"--cfg \
                 tokio_unstable\" and the tokio-console feature enabled."
            );

            Some(ConsoleLayer::builder().spawn())
        } else {
            None
        }
    }
}
