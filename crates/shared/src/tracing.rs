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

#[cfg(test)]
mod tests {
    use super::*;

    /// Installs the subscriber and emits through it.
    ///
    /// `init` panics if a global subscriber is already registered, so this is
    /// deliberately the only test in the crate that calls `init_tracing`.
    #[test]
    fn init_tracing_installs_a_global_subscriber() {
        init_tracing();

        tracing::info!(check = "value", "tracing is live");
        tracing::debug!("filtered out at the default level");
    }
}
