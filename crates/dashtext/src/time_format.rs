//! Human-readable timestamps for lists and status lines.

use chrono::DateTime;
use chrono::Datelike as _;
use chrono::Local;
use chrono::TimeZone;
use dashtext_core::Timestamp;

/// A compact date for list rows: the time for today, `Yesterday`, the weekday
/// within the last week, then the date.
pub fn short(timestamp: Timestamp) -> String {
    short_in(timestamp, &Local::now())
}

/// A phrase for status lines, such as `just now`, `12 minutes ago` or
/// `yesterday at 9:05 AM`.
pub fn relative(timestamp: Timestamp) -> String {
    relative_in(timestamp, &Local::now())
}

fn to_local<Tz: TimeZone>(timestamp: Timestamp, now: &DateTime<Tz>) -> Option<DateTime<Tz>> {
    DateTime::from_timestamp_millis(timestamp.as_millis())
        .map(|utc| utc.with_timezone(&now.timezone()))
}

fn days_between<Tz: TimeZone>(earlier: &DateTime<Tz>, later: &DateTime<Tz>) -> i64 {
    (later.date_naive() - earlier.date_naive()).num_days()
}

fn short_in<Tz: TimeZone>(timestamp: Timestamp, now: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let Some(time) = to_local(timestamp, now) else {
        return String::new();
    };
    match days_between(&time, now) {
        0 => time.format("%-I:%M %p").to_string(),
        1 => "Yesterday".to_owned(),
        2..=6 => time.format("%A").to_string(),
        _ if time.year() == now.year() => time.format("%b %-d").to_string(),
        _ => time.format("%b %-d, %Y").to_string(),
    }
}

fn relative_in<Tz: TimeZone>(timestamp: Timestamp, now: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let Some(time) = to_local(timestamp, now) else {
        return String::new();
    };
    let minutes = (now.clone() - time.clone()).num_minutes();
    match (minutes, days_between(&time, now)) {
        (..1, _) => "just now".to_owned(),
        (1, _) => "1 minute ago".to_owned(),
        (2..60, _) => format!("{minutes} minutes ago"),
        (_, 0) => format!("today at {}", time.format("%-I:%M %p")),
        (_, 1) => format!("yesterday at {}", time.format("%-I:%M %p")),
        _ => format!("on {}", short_in(timestamp, now)),
    }
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;

    fn at(rfc3339: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(rfc3339).expect("valid timestamp")
    }

    fn stamp(rfc3339: &str) -> Timestamp {
        Timestamp::from_millis(at(rfc3339).timestamp_millis())
    }

    const NOW: &str = "2026-09-29T15:30:00-05:00";

    #[test]
    fn short_dates_widen_with_age() {
        let now = at(NOW);
        assert_eq!(
            short_in(stamp("2026-09-29T09:05:00-05:00"), &now),
            "9:05 AM"
        );
        assert_eq!(
            short_in(stamp("2026-09-28T23:59:00-05:00"), &now),
            "Yesterday"
        );
        assert_eq!(
            short_in(stamp("2026-09-24T12:00:00-05:00"), &now),
            "Thursday"
        );
        assert_eq!(short_in(stamp("2026-03-02T12:00:00-05:00"), &now), "Mar 2");
        assert_eq!(
            short_in(stamp("2025-12-31T12:00:00-05:00"), &now),
            "Dec 31, 2025"
        );
    }

    #[test]
    fn short_dates_use_the_local_calendar_day() {
        // 01:00 UTC on the 29th is still the 28th five hours west.
        let now = at(NOW);
        assert_eq!(
            short_in(stamp("2026-09-29T01:00:00+00:00"), &now),
            "Yesterday"
        );
    }

    #[test]
    fn relative_phrases() {
        let now = at(NOW);
        assert_eq!(
            relative_in(stamp("2026-09-29T15:29:40-05:00"), &now),
            "just now"
        );
        assert_eq!(
            relative_in(stamp("2026-09-29T15:29:00-05:00"), &now),
            "1 minute ago"
        );
        assert_eq!(
            relative_in(stamp("2026-09-29T15:18:00-05:00"), &now),
            "12 minutes ago"
        );
        assert_eq!(
            relative_in(stamp("2026-09-29T08:07:00-05:00"), &now),
            "today at 8:07 AM"
        );
        assert_eq!(
            relative_in(stamp("2026-09-28T21:00:00-05:00"), &now),
            "yesterday at 9:00 PM"
        );
        assert_eq!(
            relative_in(stamp("2026-09-02T21:00:00-05:00"), &now),
            "on Sep 2"
        );
    }

    #[test]
    fn future_timestamps_read_as_now() {
        // Clock skew between devices must not produce "-3 minutes ago".
        let now = at(NOW);
        assert_eq!(
            relative_in(stamp("2026-09-29T15:33:00-05:00"), &now),
            "just now"
        );
    }
}
