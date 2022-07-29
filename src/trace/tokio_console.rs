#![cfg(feature = "tokio-console")]
use clap::Parser;
use console_subscriber::ConsoleLayer;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
pub struct Options {
    /// Start a tokio-console server on `http://127.0.0.1:6669/`.
    #[clap(long)]
    #[cfg(feature = "tokio-console")]
    pub tokio_console: bool,
}

impl Options {
    pub fn into_layer<S>(self) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        if self.tokio_console {
            // TODO: Remove when <https://github.com/tokio-rs/tokio/issues/4114> resolves
            // TODO: Configure server addr.
            // TODO: Manage server thread.
            assert!(
                cfg!(tokio_unstable),
                "Enabling --tokio-console requires a build with RUSTFLAGS=\"--cfg \
                 tokio_unstable\" and the tokio-console feature enabled."
            );

            Some(ConsoleLayer::builder().spawn())
        } else {
            None
        }
    }
}
