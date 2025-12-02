use std::{env, sync::Arc};

use init_tracing_opentelemetry::resource::DetectResource;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    logs::{BatchLogProcessor, SdkLogger, SdkLoggerProvider},
};
use pyroscope::{PyroscopeAgent, pyroscope::PyroscopeAgentRunning};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use tracing::{error, info};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
type OtelBridgeLayer = OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>;

pub fn init_pyroscope_agent(
    service_name: &str,
    git_sha: &str,
    is_production: bool,
) -> Option<PyroscopeAgent<PyroscopeAgentRunning>> {
    let server_url =
        env::var("PYROSCOPE_SERVER_URL").unwrap_or_else(|_| "".to_string());
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

pub struct OtelLoggingLayer {
    layer: OtelBridgeLayer,
    guard: OtelLoggerGuard,
}

impl OtelLoggingLayer {
    pub fn into_parts(self) -> (OtelBridgeLayer, OtelLoggerGuard) {
        (self.layer, self.guard)
    }
}

pub struct OtelLoggerGuard {
    provider: Arc<SdkLoggerProvider>,
}

impl OtelLoggerGuard {
    fn new(provider: Arc<SdkLoggerProvider>) -> Self {
        Self { provider }
    }
}

impl Drop for OtelLoggerGuard {
    fn drop(&mut self) {
        if let Err(err) = self.provider.shutdown() {
            error!(
                target = "otel::logs",
                "Failed to shutdown OTLP log exporter: {}", err
            );
        }
    }
}

pub fn init_otlp_logging_layer() -> Result<Option<OtelLoggingLayer>, BoxError> {
    if !otlp_logs_enabled() {
        info!(
            target = "otel::logs",
            "OTLP log exporter disabled (OTEL_LOGS_EXPORTER)"
        );
        return Ok(None);
    }

    let resource = build_resource();
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(|err| -> BoxError { Box::new(err) })?;

    let batch_processor = BatchLogProcessor::builder(exporter).build();

    let provider = Arc::new(
        SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_log_processor(batch_processor)
            .build(),
    );

    let layer = OpenTelemetryTracingBridge::new(provider.as_ref());
    let guard = OtelLoggerGuard::new(provider);

    info!(target = "otel::logs", "OTLP log exporter enabled");

    Ok(Some(OtelLoggingLayer { layer, guard }))
}

fn otlp_logs_enabled() -> bool {
    match env::var("OTEL_LOGS_EXPORTER") {
        Ok(value) => {
            let mut has_otlp = false;
            let mut saw_entry = false;

            for entry in value
                .split(',')
                .map(|part| part.trim().to_ascii_lowercase())
            {
                if entry.is_empty() {
                    continue;
                }

                saw_entry = true;

                if entry == "none" {
                    return false;
                }

                if entry == "otlp" {
                    has_otlp = true;
                }
            }

            if !saw_entry { true } else { has_otlp }
        }
        Err(_) => true,
    }
}

fn build_resource() -> Resource {
    DetectResource::default()
        .with_fallback_service_name(env!("CARGO_PKG_NAME"))
        .with_fallback_service_version(env!("CARGO_PKG_VERSION"))
        .build()
}
