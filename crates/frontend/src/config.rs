//! Frontend configuration constants.

/// Base URL for the backend API.
///
/// The value is read from the `API_BASE_URL` **compile-time** environment
/// variable (e.g. set via a Docker build arg or `export API_BASE_URL=…`
/// before running `trunk build`).
///
/// When the variable is absent the local-development default is used so that
/// plain `trunk serve` continues to work without any extra setup.
pub(crate) const API_BASE_URL: &str = if let Some(url) = option_env!("API_BASE_URL") {
    url
} else {
    // Default for `trunk serve` (non-Docker local development).
    "http://127.0.0.1:3000"
};

/// Default lookback period (in days) shown in the UI.
pub(crate) const DEFAULT_DAYS: u32 = 30;

/// Minimum allowed lookback period (in days).
pub(crate) const MIN_DAYS: u32 = 1;

/// Maximum allowed lookback period (in days).
pub(crate) const MAX_DAYS: u32 = 365;
