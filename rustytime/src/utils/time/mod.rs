use chrono::{
    DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone, Utc,
};
use chrono_tz::Tz;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Time {
    pub human_readable: String,
    pub seconds: i64,
    pub minutes: i64,
    pub hours: i64,
    pub days: i64,
}

impl Time {
    pub fn new(human_readable: String, seconds: i64, minutes: i64, hours: i64, days: i64) -> Self {
        Time {
            human_readable,
            seconds,
            minutes,
            hours,
            days,
        }
    }
}

#[allow(dead_code)]
pub enum TimeFormat {
    NoDays,
    HourMinute,
    Full,
}

pub fn human_readable_duration(seconds: i64, format: TimeFormat) -> Time {
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    let human_readable = match format {
        TimeFormat::Full => {
            if days > 0 {
                format!(
                    "{}d {}h {}m {}s",
                    days,
                    hours % 24,
                    minutes % 60,
                    seconds % 60
                )
            } else if hours > 0 {
                format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds % 60)
            } else {
                format!("{}s", seconds)
            }
        }
        TimeFormat::NoDays => {
            if hours > 0 {
                format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds % 60)
            } else {
                format!("{}s", seconds)
            }
        }
        TimeFormat::HourMinute => {
            if hours > 0 {
                format!("{}h {}m", hours, minutes % 60)
            } else if minutes > 0 {
                format!("{}m", minutes)
            } else {
                format!("{}s", seconds)
            }
        }
    };

    Time::new(
        human_readable,
        seconds % 60,
        minutes % 60,
        match format {
            TimeFormat::NoDays => hours,
            _ => hours % 24,
        },
        match format {
            TimeFormat::NoDays => 0,
            _ => days,
        },
    )
}

#[inline(always)]
pub fn format_rfc3339(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn split_range_midpoint(start: DateTime<Utc>, end: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if end <= start {
        return None;
    }

    let duration = end - start;
    let half = duration / 2;
    if half <= Duration::zero() {
        return None;
    }

    let midpoint = start.checked_add_signed(half)?;
    if midpoint <= start || midpoint >= end {
        None
    } else {
        Some(midpoint)
    }
}

pub fn determine_range(
    period_end: DateTime<Utc>,
    cutoff: DateTime<Utc>,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let adjusted_end = period_end - Duration::nanoseconds(1);
    let month_start = adjusted_end
        .date_naive()
        .with_day(1)
        .expect("every month has a first day")
        .and_hms_opt(0, 0, 0)
        .expect("valid start of month")
        .and_utc();

    let range_start = if month_start > cutoff {
        month_start
    } else {
        cutoff
    };
    (range_start, month_start)
}

#[inline(always)]
pub fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}

/// Parse a timezone string, returning the Tz or UTC as fallback
#[inline(always)]
pub fn parse_timezone(tz_str: &str) -> Tz {
    tz_str.parse().unwrap_or(chrono_tz::UTC)
}

/// Convert a naive datetime in a timezone to UTC, handling DST transitions
pub fn local_datetime_to_utc(naive_dt: NaiveDateTime, tz: Tz) -> DateTime<Utc> {
    tz.from_local_datetime(&naive_dt)
        .earliest()
        .or_else(|| tz.from_local_datetime(&naive_dt).latest())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

/// Get the start of day (00:00:00) in the user's timezone, converted to UTC
pub fn get_day_start_utc(date: NaiveDate, tz: Tz) -> DateTime<Utc> {
    let naive_dt = date.and_hms_opt(0, 0, 0).unwrap();
    local_datetime_to_utc(naive_dt, tz)
}

/// Get the end of day (start of next day) in the user's timezone, converted to UTC
pub fn get_day_end_utc(date: NaiveDate, tz: Tz) -> DateTime<Utc> {
    let tomorrow = date.succ_opt().unwrap_or(date);
    let naive_dt = tomorrow.and_hms_opt(0, 0, 0).unwrap();
    local_datetime_to_utc(naive_dt, tz)
}

/// Get today's date in the user's timezone
pub fn get_today_in_timezone(tz: Tz) -> NaiveDate {
    Utc::now().with_timezone(&tz).date_naive()
}

/// Get the start of the week (Monday) for a given date
pub fn get_week_start_date(date: NaiveDate) -> NaiveDate {
    let days_from_monday = date.weekday().num_days_from_monday() as i64;
    date - chrono::Duration::days(days_from_monday)
}

/// Get the start of the month for a given date
pub fn get_month_start_date(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
}

#[cfg(test)]
mod tests;
