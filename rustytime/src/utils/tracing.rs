use std::env;

use pyroscope::{PyroscopeAgent, pyroscope::PyroscopeAgentRunning};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use tracing::{error, info};

pub fn init_pyroscope_agent(
    service_name: &str,
    git_sha: &str,
    is_production: bool,
) -> Option<PyroscopeAgent<PyroscopeAgentRunning>> {
    let server_url =
        env::var("PYROSCOPE_SERVER_URL").unwrap_or_else(|_| "http://localhost:4040".to_string());
    if server_url.trim().is_empty() {
        info!("⚠️  Pyroscope profiler disabled (PYROSCOPE_SERVER_URL not set)");
        return None;
    }

    let sample_rate = env::var("PYROSCOPE_SAMPLE_RATE")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(99);

    let application = service_name.to_string();

    match PyroscopeAgent::builder(server_url, application)
        .backend(pprof_backend(PprofConfig::new().sample_rate(sample_rate)))
        .tags(vec![
            ("env", if is_production { "prod" } else { "dev" }),
            ("git_sha", git_sha),
        ])
        .build()
        .and_then(|agent| agent.start())
    {
        Ok(agent) => {
            info!(sample_rate = sample_rate, "✅ Pyroscope profiler started");
            Some(agent)
        }
        Err(err) => {
            error!("❌ Failed to start Pyroscope profiler: {}", err);
            None
        }
    }
}
