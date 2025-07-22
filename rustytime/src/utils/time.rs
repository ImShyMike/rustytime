#[allow(dead_code)]
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

pub fn human_readable_duration(seconds: i64) -> Time {
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    let human_readable = if days > 0 {
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
    };

    Time::new(human_readable, seconds % 60, minutes % 60, hours % 24, days)
}
