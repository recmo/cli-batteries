use eyre::Result as EyreResult;
use once_cell::sync::Lazy;
use tokio::sync::watch::{self, Receiver, Sender};
use tracing::{error, info};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(not(unix))]
use tokio::signal::ctrl_c;

static NOTIFY: Lazy<(Sender<bool>, Receiver<bool>)> = Lazy::new(|| watch::channel(false));

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

pub fn shutdown() {
    NOTIFY.0.send(true).unwrap();
}

pub fn is_shutting_down() -> bool {
    *NOTIFY.1.borrow()
}

/// Create a (cancellable) future that waits for a signal to shutdown the app.
#[allow(clippy::module_name_repetitions)]
pub async fn await_shutdown() {
    if is_shutting_down() {
        return;
    }
    NOTIFY.1.clone().changed().await.unwrap();
}

#[cfg(unix)]
#[allow(clippy::module_name_repetitions)]
async fn signal_shutdown() -> EyreResult<()> {
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

#[cfg(not(unix))]
#[allow(clippy::module_name_repetitions)]
async fn signal_shutdown() -> EyreResult<()> {
    ctrl_c().await?;
    info!("Ctrl-C received, shutting down");
    Ok(())
}
