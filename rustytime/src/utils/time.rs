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

#[cfg(test)]
mod tests {
    use super::*;

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
}
