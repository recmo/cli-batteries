use crate::shutdown::await_shutdown;
use std::time::{Duration, Instant};
use tokio::time::{interval, MissedTickBehavior};
use tracing::info;

pub async fn heartbeat() {
    let start = Instant::now();

    let mut interval = interval(Duration::from_secs(5 * 60));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval.reset(); // Skip immediate first tick

    loop {
        tokio::select! {
            _ = await_shutdown() => break,
            _ = interval.tick() => {},
        };

        // Measure uptime
        let uptime = start.elapsed();

        info!(?uptime, "Heartbeat");

        // FEATURE: Log Tokio metrics once API is available.
    }
}
