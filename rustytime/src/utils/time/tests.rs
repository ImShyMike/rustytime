use super::*;
use chrono::{NaiveDate, TimeZone};

const SECONDS: i64 = 90061;

#[test]
fn formats_duration_with_full_precision() {
    let time_obj = human_readable_duration(SECONDS, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "1d 1h 1m 1s");
}

#[test]
fn formats_duration_without_day_component() {
    let time_obj = human_readable_duration(SECONDS, TimeFormat::NoDays);
    assert_eq!(time_obj.human_readable, "25h 1m 1s");
}

#[test]
fn formats_duration_for_hour_minute_display() {
    let time_obj = human_readable_duration(SECONDS, TimeFormat::HourMinute);
    assert_eq!(time_obj.human_readable, "25h 1m");
}

#[test]
fn formats_duration_zero_seconds() {
    let time_obj = human_readable_duration(0, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "0s");
    assert_eq!(time_obj.seconds, 0);
    assert_eq!(time_obj.minutes, 0);
    assert_eq!(time_obj.hours, 0);
    assert_eq!(time_obj.days, 0);
}

#[test]
fn formats_duration_only_seconds() {
    let time_obj = human_readable_duration(45, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "45s");
    assert_eq!(time_obj.seconds, 45);
    assert_eq!(time_obj.minutes, 0);
    assert_eq!(time_obj.hours, 0);
    assert_eq!(time_obj.days, 0);
}

#[test]
fn formats_duration_only_minutes() {
    let time_obj = human_readable_duration(125, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "2m 5s");
    assert_eq!(time_obj.seconds, 5);
    assert_eq!(time_obj.minutes, 2);
    assert_eq!(time_obj.hours, 0);
    assert_eq!(time_obj.days, 0);
}

#[test]
fn formats_duration_only_hours() {
    let time_obj = human_readable_duration(7265, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "2h 1m 5s");
    assert_eq!(time_obj.seconds, 5);
    assert_eq!(time_obj.minutes, 1);
    assert_eq!(time_obj.hours, 2);
    assert_eq!(time_obj.days, 0);
}

#[test]
fn formats_duration_multiple_days() {
    let time_obj = human_readable_duration(259261, TimeFormat::Full);
    assert_eq!(time_obj.human_readable, "3d 0h 1m 1s");
    assert_eq!(time_obj.seconds, 1);
    assert_eq!(time_obj.minutes, 1);
    assert_eq!(time_obj.hours, 0);
    assert_eq!(time_obj.days, 3);
}

#[test]
fn formats_duration_verifies_time_struct_fields() {
    let time_obj = human_readable_duration(SECONDS, TimeFormat::Full);
    assert_eq!(time_obj.seconds, 1);
    assert_eq!(time_obj.minutes, 1);
    assert_eq!(time_obj.hours, 1);
    assert_eq!(time_obj.days, 1);
}

#[test]
fn formats_duration_no_days_verifies_time_struct_fields() {
    let time_obj = human_readable_duration(SECONDS, TimeFormat::NoDays);
    assert_eq!(time_obj.seconds, 1);
    assert_eq!(time_obj.minutes, 1);
    assert_eq!(time_obj.hours, 25);
    assert_eq!(time_obj.days, 0);
}

#[test]
fn format_rfc3339_basic() {
    let time = Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 45).unwrap();
    let formatted = format_rfc3339(time);
    assert_eq!(formatted, "2024-06-15T12:30:45.000Z");
}

#[test]
fn format_rfc3339_millisecond_precision() {
    let time = Utc
        .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
        .unwrap()
        .checked_add_signed(Duration::milliseconds(123))
        .unwrap();
    let formatted = format_rfc3339(time);
    assert_eq!(formatted, "2024-01-01T00:00:00.123Z");
}

#[test]
fn split_range_midpoint_valid_range() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 2, 0, 0).unwrap();
    let midpoint = split_range_midpoint(start, end);
    assert_eq!(
        midpoint,
        Some(Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap())
    );
}

#[test]
fn split_range_midpoint_end_before_start() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 2, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(split_range_midpoint(start, end), None);
}

#[test]
fn split_range_midpoint_end_equals_start() {
    let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(split_range_midpoint(time, time), None);
}

#[test]
fn split_range_midpoint_very_small_range() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = start.checked_add_signed(Duration::nanoseconds(1)).unwrap();
    assert_eq!(split_range_midpoint(start, end), None);
}

#[test]
fn split_range_midpoint_large_range() {
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
    let midpoint = split_range_midpoint(start, end).unwrap();
    assert!(midpoint > start);
    assert!(midpoint < end);
}

#[test]
fn determine_range_month_start_after_cutoff() {
    let period_end = Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap();
    let cutoff = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let (range_start, month_start) = determine_range(period_end, cutoff);
    assert_eq!(
        month_start,
        Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(range_start, month_start);
}

#[test]
fn determine_range_month_start_before_cutoff() {
    let period_end = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    let cutoff = Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap();
    let (range_start, month_start) = determine_range(period_end, cutoff);
    assert_eq!(
        month_start,
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(range_start, cutoff);
}

#[test]
fn determine_range_at_month_boundary() {
    let period_end = Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap();
    let cutoff = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    let (range_start, month_start) = determine_range(period_end, cutoff);
    assert_eq!(
        month_start,
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(range_start, month_start);
}

#[test]
fn get_week_start_monday() {
    let monday = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap();
    assert_eq!(get_week_start(monday), monday);
}

#[test]
fn get_week_start_sunday() {
    let sunday = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();
    let expected_monday = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap();
    assert_eq!(get_week_start(sunday), expected_monday);
}

#[test]
fn get_week_start_mid_week() {
    let wednesday = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
    let expected_monday = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap();
    assert_eq!(get_week_start(wednesday), expected_monday);
}
