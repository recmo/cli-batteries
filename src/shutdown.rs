use once_cell::sync::Lazy;
use tokio::sync::watch::{self, Receiver, Sender};

#[cfg(feature = "signals")]
use eyre::Result as EyreResult;
#[cfg(feature = "signals")]
use tracing::{error, info};

static NOTIFY: Lazy<(Sender<bool>, Receiver<bool>)> = Lazy::new(|| watch::channel(false));

/// Send the signal to shutdown the program.
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn shutdown() {
    // Does not fail because the channel never closes.
    NOTIFY.0.send(true).unwrap();
}

/// Reset the shutdown signal so it can be triggered again.
///
/// This is only useful for testing. Strange things can happen to any existing
/// `await_shutdown()` futures.
#[cfg(feature = "mock-shutdown")]
#[allow(clippy::missing_panics_doc)] // Never panics
#[allow(clippy::module_name_repetitions)] // Never panics
pub fn reset_shutdown() {
    // Does not fail because the channel never closes.
    NOTIFY.0.send(false).unwrap();
}

/// Are we currently shutting down?
#[must_use]
pub fn is_shutting_down() -> bool {
    *NOTIFY.1.borrow()
}

/// Wait for the program to shutdown.
///
/// Resolves immediately if the program is already shutting down.
/// The resulting future is safe to cancel by dropping.
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::missing_panics_doc)]
pub async fn await_shutdown() {
    let mut watch = NOTIFY.1.clone();
    if *watch.borrow_and_update() {
        return;
    }
    // Does not fail because the channel never closes.
    watch.changed().await.unwrap();
}

#[cfg(feature = "signals")]
pub fn watch_signals() {
    tokio::spawn({
        async move {
            signal_shutdown()
                .await
                .map_err(|err| error!("Error handling Ctrl-C: {}", err))
                .unwrap();
            shutdown();
        }
    });
}

#[cfg(all(unix, feature = "signals"))]
#[allow(clippy::module_name_repetitions)]
async fn signal_shutdown() -> EyreResult<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let sigint = signal(SignalKind::interrupt())?;
    let sigterm = signal(SignalKind::terminate())?;
    tokio::pin!(sigint);
    tokio::pin!(sigterm);
    tokio::select! {
        _ = sigint.recv() => { info!("SIGINT received, shutting down"); }
        _ = sigterm.recv() => { info!("SIGTERM received, shutting down"); }
    };
    Ok(())
}

#[cfg(all(not(unix), feature = "signals"))]
#[allow(clippy::module_name_repetitions)]
async fn signal_shutdown() -> EyreResult<()> {
    use tokio::signal::ctrl_c;

    ctrl_c().await?;
    info!("Ctrl-C received, shutting down");
    Ok(())
}
