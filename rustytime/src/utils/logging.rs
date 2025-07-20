use tracing_subscriber::EnvFilter;

/// initialize tracing subscriber with environment filter
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();
}
