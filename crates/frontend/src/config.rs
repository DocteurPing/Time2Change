//! Frontend configuration constants.

/// Base URL for the backend API.
///
/// Keep this in one place so environments can be swapped easily later.
pub(crate) const API_BASE_URL: &str = "http://127.0.0.1:3000";

/// Default lookback period (in days) shown in the UI.
pub(crate) const DEFAULT_DAYS: u32 = 30;

/// Minimum allowed lookback period (in days).
pub(crate) const MIN_DAYS: u32 = 1;

/// Maximum allowed lookback period (in days).
pub(crate) const MAX_DAYS: u32 = 365;
