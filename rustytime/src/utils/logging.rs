use sentry::{ClientInitGuard, ClientOptions};
use std::env;
use tracing_subscriber::EnvFilter;

use tracing::{error, info};

/// Initialize tracing subscriber with environment filter
#[inline(always)]
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();
}

pub fn init_sentry() -> Option<ClientInitGuard> {
    let dsn = match env::var("SENTRY_DSN") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            info!("Sentry disabled; `SENTRY_DSN` not set");
            return None;
        }
    };

    let mut options = ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: env::var("SENTRY_TRACES_SAMPLE_RATE")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.0),
        send_default_pii: env::var("SENTRY_SEND_DEFAULT_PII")
            .ok()
            .and_then(|value| parse_env_bool(&value))
            .unwrap_or(true),
        enable_logs: true,
        ..ClientOptions::default()
    };

    options.dsn = match dsn.parse() {
        Ok(parsed) => Some(parsed),
        Err(err) => {
            error!("Invalid SENTRY_DSN provided: {}", err);
            return None;
        }
    };

    let guard = sentry::init(options);
    info!("✅ Sentry initialized");
    Some(guard)
}

fn parse_env_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
