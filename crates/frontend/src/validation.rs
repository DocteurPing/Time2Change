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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_currency() {
        let result = validate_analysis_input("", "USD", "30");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Please select both base and quote currencies."
        );
    }

    #[test]
    fn rejects_same_currency_pair() {
        let result = validate_analysis_input("EUR", "EUR", "30");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Base and quote currency must be different."
        );
    }

    #[test]
    fn rejects_non_numeric_days() {
        let result = validate_analysis_input("EUR", "USD", "abc");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("`days` must be a number between {MIN_DAYS} and {MAX_DAYS}.")
        );
    }

    #[test]
    fn rejects_days_below_min() {
        let below_min = MIN_DAYS.saturating_sub(1).to_string();
        let result = validate_analysis_input("EUR", "USD", &below_min);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("`days` must be a number between {MIN_DAYS} and {MAX_DAYS}.")
        );
    }

    #[test]
    fn rejects_days_above_max() {
        let above_max = (MAX_DAYS + 1).to_string();
        let result = validate_analysis_input("EUR", "USD", &above_max);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("`days` must be a number between {MIN_DAYS} and {MAX_DAYS}.")
        );
    }

    #[test]
    fn accepts_valid_input() {
        let result = validate_analysis_input("EUR", "USD", "30");
        assert_eq!(result, Ok(30));
    }
}
