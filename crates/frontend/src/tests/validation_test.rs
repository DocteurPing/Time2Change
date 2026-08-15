use crate::config::{MAX_DAYS, MIN_DAYS};
use crate::validation::validate_analysis_input;

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
