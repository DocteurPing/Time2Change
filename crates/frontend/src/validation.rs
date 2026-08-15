//! Input validation for the frontend form.

use crate::config::{MAX_DAYS, MIN_DAYS};

/// Validates user-provided form input and returns the parsed `days` value.
///
/// On success, returns the parsed `days`.
/// On failure, returns a user-facing error message.
pub(crate) fn validate_analysis_input(
    base: &str,
    quote: &str,
    days_raw: &str,
) -> Result<u32, String> {
    let base = base.trim();
    let quote = quote.trim();

    if base.is_empty() || quote.is_empty() {
        return Err("Please select both base and quote currencies.".to_owned());
    }

    if base == quote {
        return Err("Base and quote currency must be different.".to_owned());
    }

    let parsed_days = days_raw
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("`days` must be a number between {MIN_DAYS} and {MAX_DAYS}."))?;

    if !(MIN_DAYS..=MAX_DAYS).contains(&parsed_days) {
        return Err(format!(
            "`days` must be a number between {MIN_DAYS} and {MAX_DAYS}."
        ));
    }

    Ok(parsed_days)
}
