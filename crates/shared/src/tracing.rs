use tracing_subscriber::{EnvFilter, fmt};

/// Initializes the tracing subscriber with a default filter.
pub fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
