use crate::database::dao::last_update::UpdatableElement::EventsNotification;
use crate::database::dao::last_update::{get_last_update, set_last_update};
use crate::database::error::DatabaseError;
#[cfg(not(test))]
use crate::error::ApplicationError;
use crate::error::Result;
use crate::notification::error::NotificationError;
use crate::notification::error::NotificationError::MissingEventsIncomingAddresses;
#[cfg(not(test))]
use crate::tools::email::send_email;
use crate::tools::env_args::retrieve_expected_arg_value;
#[cfg(not(test))]
use crate::tools::log_message_and_return;
#[cfg(not(test))]
use crate::tools::web::build_client;
#[cfg(not(test))]
use crate::web::error::WebError::ConnectionFailed;
use chrono::{Days, NaiveDate, NaiveDateTime};
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::BTreeSet;
#[cfg(test)]
use std::fs;
use std::string::String;
use tera::{Context, Tera};

#[cfg(not(test))]
const FEED_URL: &str = "https://monocycle.info/events/feed"; // TODO: pass in the config
const EVENTS_INCOMING_ADDRESSES: &str = "--events-incoming-addresses";
const DELAY_BETWEEN_EXECUTIONS_IN_DAYS: u8 = 7; // TODO: pass in the config
const NUMBER_OF_EXECUTIONS_TO_EVENTS: u8 = 2; // TODO: pass in the config
const DAYS_BEFORE_EVENTS: u8 = DELAY_BETWEEN_EXECUTIONS_IN_DAYS * NUMBER_OF_EXECUTIONS_TO_EVENTS;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug, Deserialize)]
struct RssFeed {
    channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    item: Vec<Event>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
struct Event {
    title: String,
    link: String,
    event_day: u8,
    event_month: u8,
    event_year: u16,
}

impl Event {
    fn get_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(
            self.event_year as i32,
            self.event_month as u32,
            self.event_day as u32,
        )
    }
}

impl PartialOrd<Self> for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.event_year
            .cmp(&other.event_year)
            .then(self.event_month.cmp(&other.event_month))
            .then(self.event_day.cmp(&other.event_day))
    }
}

/// Range of event dates to notify about: `after` is excluded, `up_to` is included.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct NotificationWindow {
    after: NaiveDate,
    up_to: NaiveDate,
}

impl NotificationWindow {
    fn contains(&self, date: NaiveDate) -> bool {
        self.after < date && date <= self.up_to
    }
}

/// Notify about incoming events if the previous notification is old enough.
///
/// The date of the last notification is persisted, so that restarting the app neither skips
/// events nor sends the same notification twice.
/// Returns whether notifications have been sent.
pub async fn notify_incoming_events_if_due(pool: &DbPool) -> Result<bool> {
    let now = get_now();
    let last_notification = {
        let mut connection = get_connection(pool)?;
        get_last_update(&mut connection, &EventsNotification)?
    };
    let Some(window) =
        compute_notification_window(last_notification.map(|date| date.date()), now.date())
    else {
        debug!("Incoming events notification is not due yet");
        return Ok(false);
    };

    info!(
        "Notifying about events after {} and up to {}",
        window.after, window.up_to
    );
    let feed = get_events_feed().await?;
    let feed = deserialize_feed(&feed)?;
    let incoming_events = retrieve_only_events_in_window(&feed.channel.item, &window);
    notify_for_incoming_events(&incoming_events).await?;

    let mut connection = get_connection(pool)?;
    set_last_update(&mut connection, &EventsNotification, now)?;

    Ok(true)
}

fn get_connection(
    pool: &DbPool,
) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
    pool.get()
        .map_err(|error| DatabaseError::R2d2(error.to_string()))
}

/// Compute which events should be notified about, or `None` if the last notification is too recent.
///
/// Each notification covers the events right after the ones covered by the previous notification,
/// so that a late execution (e.g., the app was down) doesn't skip any event.
fn compute_notification_window(
    last_notification: Option<NaiveDate>,
    today: NaiveDate,
) -> Option<NotificationWindow> {
    let up_to = add_days(today, DAYS_BEFORE_EVENTS);
    let after = match last_notification {
        Some(last_notification) => {
            if today < add_days(last_notification, DELAY_BETWEEN_EXECUTIONS_IN_DAYS) {
                return None;
            }
            // Events already passed are not worth notifying about.
            add_days(last_notification, DAYS_BEFORE_EVENTS).max(today)
        }
        None => add_days(today, DAYS_BEFORE_EVENTS - DELAY_BETWEEN_EXECUTIONS_IN_DAYS),
    };

    Some(NotificationWindow { after, up_to })
}

fn add_days(date: NaiveDate, days: u8) -> NaiveDate {
    date.checked_add_days(Days::new(days as u64))
        .unwrap_or_else(|| panic!("Date {days} days after {date} should exist..."))
}

#[cfg(not(test))]
async fn get_events_feed() -> Result<String> {
    let client = build_client()?;
    let response = client
        .get(FEED_URL)
        .send()
        .await
        .map_err(log_message_and_return(
            "Connection failed...",
            ConnectionFailed,
        ))?;
    let status = response.status();
    if !status.is_success() {
        log::error!("Connection failed because of status {status}...");
        return Err(ApplicationError::from(ConnectionFailed));
    }

    let text = response.text().await.map_err(log_message_and_return(
        "Couldn't get text of response",
        ConnectionFailed,
    ))?;

    Ok(text)
}

fn deserialize_feed(feed_text: &str) -> Result<RssFeed, NotificationError> {
    Ok(quick_xml::de::from_str(feed_text)?)
}

fn retrieve_only_events_in_window<'a>(
    events: &'a [Event],
    window: &NotificationWindow,
) -> BTreeSet<&'a Event> {
    events
        .iter()
        .filter(|event| is_event_in_window(event, window))
        .collect()
}

async fn notify_for_incoming_events(events: &BTreeSet<&Event>) -> Result<(), NotificationError> {
    let recipients = retrieve_events_incoming_addresses()?;
    let recipients: Vec<&str> = recipients
        .iter()
        .map(|s| s.as_str().trim())
        .filter(|s| !s.is_empty())
        .collect();
    let tera = create_tera_renderer()?;
    for event in events {
        notify_for_incoming_event(&recipients, &tera, event).await?;
    }
    Ok(())
}

async fn notify_for_incoming_event(
    recipients: &Vec<&str>,
    tera: &Tera,
    event: &Event,
) -> Result<(), NotificationError> {
    let body = create_email_body(tera, event)?;
    #[cfg(not(test))] // We don't want to send emails in test mode.
    send_email(recipients, "Vérification de licences", &body).await?;
    #[cfg(test)]
    println!(
        "Mocking email sending [recipients: {:?}, body: {}]",
        recipients, body
    );
    Ok(())
}

fn is_event_in_window(event: &Event, window: &NotificationWindow) -> bool {
    event.get_date().is_some_and(|date| window.contains(date))
}

#[cfg(not(test))]
fn get_now() -> NaiveDateTime {
    chrono::offset::Utc::now().naive_utc()
}

fn create_email_body(tera: &Tera, event: &Event) -> Result<String, NotificationError> {
    let event_date = format!(
        "{}/{}/{}",
        event.event_day, event.event_month, event.event_year
    );

    let mut context = Context::new();
    context.insert("event_name", &event.title);
    context.insert("event_start_date", &event_date);
    context.insert("event_link", &event.link);
    let body = tera.render("incoming-event-body.html.tera", &context)?;
    Ok(body)
}

fn create_tera_renderer() -> Result<Tera, NotificationError> {
    Ok(Tera::new("public/templates/notification/*.html.tera")?)
}

fn retrieve_events_incoming_addresses() -> Result<Vec<String>, NotificationError> {
    retrieve_expected_arg_value(EVENTS_INCOMING_ADDRESSES, MissingEventsIncomingAddresses)
        .map(|addresses| addresses.split(',').map(String::from).collect())
}

#[cfg(test)]
fn get_now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2025, 1, 1)
        .and_then(|date| date.and_hms_opt(8, 0, 0))
        .expect("Should be great for testing")
}

#[cfg(test)]
async fn get_events_feed() -> Result<String> {
    Ok(fs::read_to_string("test/resources/events_feed.xml").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDRESS: &str = "email@address.com";
    const SECOND_TEST_ADDRESS: &str = "email@address.com";

    fn get_args() -> Vec<String> {
        vec![format!(
            "{EVENTS_INCOMING_ADDRESSES}={TEST_ADDRESS},{SECOND_TEST_ADDRESS}"
        )]
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn event_on(date: NaiveDate) -> Event {
        Event {
            title: "event title".to_string(),
            link: "https://event.link".to_string(),
            event_year: date.year() as u16,
            event_month: date.month() as u8,
            event_day: date.day() as u8,
        }
    }

    fn get_window() -> NotificationWindow {
        NotificationWindow {
            after: date(2025, 1, 8),
            up_to: date(2025, 1, 15),
        }
    }

    use chrono::Datelike;

    mod notify_incoming_events_if_due {
        use super::*;
        use crate::database::with_temp_database;
        use crate::tools::env_args::with_env_args;
        use rocket::futures::executor::block_on;

        #[test]
        fn notify_when_never_notified() {
            with_temp_database(|pool| {
                with_env_args(get_args(), || {
                    assert!(block_on(notify_incoming_events_if_due(&pool)).unwrap());

                    let mut connection = pool.get().unwrap();
                    assert_eq!(
                        Some(get_now()),
                        get_last_update(&mut connection, &EventsNotification).unwrap()
                    );
                })
            });
        }

        #[test]
        fn do_not_notify_twice() {
            with_temp_database(|pool| {
                with_env_args(get_args(), || {
                    assert!(block_on(notify_incoming_events_if_due(&pool)).unwrap());
                    assert!(!block_on(notify_incoming_events_if_due(&pool)).unwrap());
                })
            });
        }

        #[test]
        fn notify_when_last_notification_is_old_enough() {
            with_temp_database(|pool| {
                with_env_args(get_args(), || {
                    let last_notification = get_now()
                        .checked_sub_days(Days::new(DELAY_BETWEEN_EXECUTIONS_IN_DAYS as u64))
                        .unwrap();
                    let mut connection = pool.get().unwrap();
                    set_last_update(&mut connection, &EventsNotification, last_notification)
                        .unwrap();
                    drop(connection);

                    assert!(block_on(notify_incoming_events_if_due(&pool)).unwrap());
                })
            });
        }

        #[test]
        fn fail_without_addresses_and_do_not_record_notification() {
            with_temp_database(|pool| {
                assert!(block_on(notify_incoming_events_if_due(&pool)).is_err());

                let mut connection = pool.get().unwrap();
                assert_eq!(
                    None,
                    get_last_update(&mut connection, &EventsNotification).unwrap()
                );
            });
        }
    }

    mod compute_notification_window {
        use super::*;

        #[test]
        fn first_notification() {
            assert_eq!(
                Some(NotificationWindow {
                    after: date(2025, 1, 8),
                    up_to: date(2025, 1, 15),
                }),
                compute_notification_window(None, date(2025, 1, 1))
            );
        }

        #[test]
        fn not_due_yet() {
            assert_eq!(
                None,
                compute_notification_window(Some(date(2025, 1, 1)), date(2025, 1, 7))
            );
        }

        #[test]
        fn not_due_same_day() {
            assert_eq!(
                None,
                compute_notification_window(Some(date(2025, 1, 1)), date(2025, 1, 1))
            );
        }

        #[test]
        fn due_on_time() {
            assert_eq!(
                Some(NotificationWindow {
                    after: date(2025, 1, 15),
                    up_to: date(2025, 1, 22),
                }),
                compute_notification_window(Some(date(2025, 1, 1)), date(2025, 1, 8))
            );
        }

        #[test]
        fn late_notification_covers_skipped_days() {
            assert_eq!(
                Some(NotificationWindow {
                    after: date(2025, 1, 15),
                    up_to: date(2025, 1, 25),
                }),
                compute_notification_window(Some(date(2025, 1, 1)), date(2025, 1, 11))
            );
        }

        #[test]
        fn very_late_notification_ignores_past_events() {
            assert_eq!(
                Some(NotificationWindow {
                    after: date(2025, 3, 1),
                    up_to: date(2025, 3, 15),
                }),
                compute_notification_window(Some(date(2025, 1, 1)), date(2025, 3, 1))
            );
        }
    }

    mod deserialize_feed {
        use crate::notification::retrieve_events::{deserialize_feed, get_events_feed};

        #[async_test]
        async fn success() {
            let events_feed = get_events_feed().await.unwrap();
            let feed = deserialize_feed(&events_feed).unwrap();
            assert_eq!(10, feed.channel.item.len())
        }
    }

    mod retrieve_only_events_in_window {
        use super::*;

        #[test]
        fn success() {
            let event_in_window = event_on(date(2025, 1, 10));
            let events = vec![
                event_in_window.clone(),
                event_on(date(2025, 1, 8)),
                event_on(date(2025, 1, 16)),
            ];
            let expected_events = BTreeSet::from([&event_in_window]);
            assert_eq!(
                expected_events,
                retrieve_only_events_in_window(&events, &get_window())
            );
        }
    }

    mod notify_for_incoming_events {
        use super::*;
        use crate::tools::env_args::with_env_args;
        use rocket::futures::executor::block_on;

        #[test]
        fn success() {
            let event = event_on(date(2025, 1, 15));
            let events = BTreeSet::from_iter(vec![&event]);
            with_env_args(get_args(), || {
                block_on(notify_for_incoming_events(&events)).unwrap();
            });
        }
    }

    mod is_event_in_window {
        use super::*;

        #[test]
        fn should_be_in_window_last_day() {
            assert!(is_event_in_window(
                &event_on(date(2025, 1, 15)),
                &get_window()
            ));
        }

        #[test]
        fn should_be_in_window_first_day() {
            assert!(is_event_in_window(
                &event_on(date(2025, 1, 9)),
                &get_window()
            ));
        }

        #[test]
        fn should_not_be_in_window_too_soon() {
            assert!(!is_event_in_window(
                &event_on(date(2025, 1, 8)),
                &get_window()
            ));
        }

        #[test]
        fn should_not_be_in_window_too_late() {
            assert!(!is_event_in_window(
                &event_on(date(2025, 1, 16)),
                &get_window()
            ));
        }

        #[test]
        fn should_not_be_in_window_invalid_date() {
            let mut event = event_on(date(2025, 1, 10));
            event.event_month = 13;
            assert!(!is_event_in_window(&event, &get_window()));
        }
    }

    mod create_email_body {
        use super::*;

        #[test]
        fn success() {
            let tera = create_tera_renderer().unwrap();
            let event = event_on(date(2025, 11, 1));
            create_email_body(&tera, &event).unwrap();
        }
    }

    mod create_tera_renderer {
        use super::*;

        #[test]
        fn success() {
            create_tera_renderer().unwrap();
        }
    }

    mod retrieve_events_incoming_addresses {
        use crate::notification::retrieve_events::retrieve_events_incoming_addresses;
        use crate::notification::retrieve_events::tests::{
            SECOND_TEST_ADDRESS, TEST_ADDRESS, get_args,
        };
        use crate::tools::env_args::with_env_args;

        #[test]
        fn success() {
            let recipients =
                with_env_args(get_args(), || retrieve_events_incoming_addresses().unwrap());

            assert_eq!(
                vec![TEST_ADDRESS.to_string(), SECOND_TEST_ADDRESS.to_string()],
                recipients
            );
        }

        #[test]
        #[should_panic(expected = "MissingEventsIncomingAddresses")]
        fn fail_when_no_address() {
            retrieve_events_incoming_addresses().unwrap();
        }
    }
}
