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
use chrono::{Days, NaiveDate};
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

#[allow(unused)]
pub async fn retrieve_events() -> Result<()> {
    let feed = get_events_feed().await?;
    let feed = deserialize_feed(&feed)?;
    let incoming_events = retrieve_only_events_in_range(&feed.channel.item);
    notify_for_incoming_events(&incoming_events).await?;

    Ok(())
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

fn retrieve_only_events_in_range(events: &[Event]) -> BTreeSet<&Event> {
    events
        .iter()
        .filter(|event| is_event_in_range(event))
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

fn is_event_in_range(event: &Event) -> bool {
    if let Some(event_date) = event.get_date() {
        let now = get_now();
        let date_in_n_executions = now
            .checked_add_days(Days::new((DAYS_BEFORE_EVENTS) as u64))
            .unwrap_or_else(|| panic!("Date in {DAYS_BEFORE_EVENTS} days should exist..."));
        let date_in_n_minus_1_executions = now
            .checked_add_days(Days::new(
                (DAYS_BEFORE_EVENTS - DELAY_BETWEEN_EXECUTIONS_IN_DAYS) as u64,
            ))
            .unwrap_or_else(|| {
                panic!(
                    "Date in {} days should exist...",
                    DAYS_BEFORE_EVENTS - DELAY_BETWEEN_EXECUTIONS_IN_DAYS
                )
            });
        return date_in_n_minus_1_executions < event_date && event_date <= date_in_n_executions;
    }

    false
}

#[cfg(not(test))]
fn get_now() -> NaiveDate {
    chrono::offset::Utc::now().date_naive()
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
fn get_now() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 1, 1).expect("Should be great for testing")
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

    fn get_event_in_range_last_day() -> Event {
        let title = "event title".to_string();
        let link = "https://event.link".to_string();
        let event_year = 2025;
        let event_month = 1;
        let event_day = 1 + DAYS_BEFORE_EVENTS;
        Event {
            title,
            link,
            event_year,
            event_month,
            event_day,
        }
    }

    fn get_event_in_range_first_day() -> Event {
        let title = "event title".to_string();
        let link = "https://event.link".to_string();
        let event_year = 2025;
        let event_month = 1;
        let event_day = 1 + DAYS_BEFORE_EVENTS - DELAY_BETWEEN_EXECUTIONS_IN_DAYS + 1;
        Event {
            title,
            link,
            event_year,
            event_month,
            event_day,
        }
    }

    fn get_event_too_late() -> Event {
        let title = "event title".to_string();
        let link = "https://event.link".to_string();
        let event_year = 2025;
        let event_month = 1;
        let event_day = 1 + DAYS_BEFORE_EVENTS + 1;
        Event {
            title,
            link,
            event_year,
            event_month,
            event_day,
        }
    }

    fn get_event_too_soon() -> Event {
        let title = "event title".to_string();
        let link = "https://event.link".to_string();
        let event_year = 2025;
        let event_month = 1;
        let event_day = 1 + DAYS_BEFORE_EVENTS - DELAY_BETWEEN_EXECUTIONS_IN_DAYS - 1;
        Event {
            title,
            link,
            event_year,
            event_month,
            event_day,
        }
    }

    mod retrieve_events {
        use crate::notification::retrieve_events::retrieve_events;
        use crate::notification::retrieve_events::tests::get_args;
        use crate::tools::env_args::with_env_args;
        use rocket::futures::executor::block_on;

        #[test]
        fn success() {
            with_env_args(get_args(), || {
                block_on(retrieve_events()).unwrap();
            });
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

    mod retrieve_only_events_in_range {
        use super::*;

        #[test]
        fn success() {
            let event_in_range_first_day = get_event_in_range_first_day();
            let events = vec![event_in_range_first_day.clone(), get_event_too_soon()];
            let expected_events = BTreeSet::from([&event_in_range_first_day]);
            assert_eq!(expected_events, retrieve_only_events_in_range(&events));
        }
    }

    mod notify_for_incoming_events {
        use super::*;
        use crate::tools::env_args::with_env_args;
        use rocket::futures::executor::block_on;

        #[test]
        fn success() {
            let title = "event title".to_string();
            let link = "https://event.link".to_string();
            let event_year = 2025;
            let event_month = 1;
            let event_day = 1 + DAYS_BEFORE_EVENTS;
            let event = Event {
                title,
                link,
                event_year,
                event_month,
                event_day,
            };
            let events = BTreeSet::from_iter(vec![&event]);
            with_env_args(get_args(), || {
                block_on(notify_for_incoming_events(&events)).unwrap();
            });
        }
    }

    mod is_event_in_range {
        use super::*;

        #[test]
        fn should_be_in_range_last_day() {
            let event = get_event_in_range_last_day();
            assert!(is_event_in_range(&event));
        }

        #[test]
        fn should_be_in_range_first_day() {
            let event = get_event_in_range_first_day();
            assert!(is_event_in_range(&event));
        }

        #[test]
        fn should_not_be_in_range_too_soon() {
            let event = get_event_too_soon();
            assert!(!is_event_in_range(&event));
        }

        #[test]
        fn should_not_be_in_range_too_late() {
            let event = get_event_too_late();
            assert!(!is_event_in_range(&event));
        }
    }

    mod create_email_body {
        use super::*;

        #[test]
        fn success() {
            let tera = create_tera_renderer().unwrap();
            let title = "event title".to_string();
            let link = "https://event.link".to_string();
            let event_year = 2025;
            let event_month = 11;
            let event_day = 1;
            let event = Event {
                title,
                link,
                event_year,
                event_month,
                event_day,
            };
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
