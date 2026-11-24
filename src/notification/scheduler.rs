use crate::notification::retrieve_events::{DbPool, notify_incoming_events_if_due};
use rocket::fairing::AdHoc;
use rocket::tokio;
use rocket::tokio::time::MissedTickBehavior;
use std::time::Duration;

/// How often to check whether incoming events should be notified.
/// The actual notification frequency is driven by the date of the last notification.
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Fairing starting a background task that periodically notifies about incoming events.
pub fn events_notification_fairing(pool: DbPool) -> AdHoc {
    AdHoc::on_liftoff("Incoming events notification", move |_| {
        Box::pin(async move {
            tokio::spawn(check_periodically(pool));
        })
    })
}

async fn check_periodically(pool: DbPool) {
    let mut interval = tokio::time::interval(CHECK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        // First tick completes immediately, so that a check happens right after (re)start.
        interval.tick().await;
        match notify_incoming_events_if_due(&pool).await {
            Ok(true) => info!("Incoming events notification sent"),
            Ok(false) => {}
            Err(error) => error!("Failed to notify about incoming events: {error:?}"),
        }
    }
}
