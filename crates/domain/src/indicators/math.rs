use rust_decimal::{Decimal, MathematicalOps, dec};

use crate::types::exchange_rate::ExchangeRate;

/// Returns the arithmetic mean of the provided decimal values.
///
/// When `values` is empty, this function returns `None`.
#[must_use]
pub fn average(values: &[Decimal]) -> Option<Decimal> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<Decimal>() / Decimal::from(values.len()))
    }
}

/// Computes a rolling arithmetic mean over `values` using the given `window`.
///
/// The returned vector always has the same length as `values`.
/// Entries that do not yet have enough preceding values to fill the window are
/// represented as `None`.
///
/// If `window` is `0` or larger than the input length, the result contains only
/// `None` values.
#[must_use]
pub fn rolling_average(values: &[Decimal], window: usize) -> Vec<Option<Decimal>> {
    if window == 0 || window > values.len() {
        return vec![None; values.len()];
    }

    let mut result = vec![None; window - 1];
    let mut sum: Decimal = values[..window].iter().sum();

    result.push(Some(sum / Decimal::from(window)));

    for i in window..values.len() {
        sum += values[i];
        sum -= values[i - window];
        result.push(Some(sum / Decimal::from(window)));
    }

    result
}

/// Returns the population standard deviation of the provided decimal values.
///
/// This function computes variance using `n` as the divisor rather than
/// `n - 1`.
///
/// When `values` is empty, this function returns `None`.
#[must_use]
pub fn standard_deviation(values: &[Decimal]) -> Option<Decimal> {
    if values.is_empty() {
        None
    } else {
        let mean = average(values)?;
        let variance = values
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(values.len());
        variance.sqrt()
    }
}

/// Returns the normalized position of `current` within the inclusive range
/// defined by `low` and `high`.
///
/// The result is typically in the range `0..=1` when `current` lies between
/// `low` and `high`, but values outside that interval are possible if
/// `current` falls outside the range.
///
/// Returns `None` when `high == low`, because the range width is zero.
#[must_use]
pub fn range_position(current: Decimal, high: Decimal, low: Decimal) -> Option<Decimal> {
    if high == low {
        None
    } else {
        Some((current - low) / (high - low))
    }
}

/// Computes the z-score of `current` relative to `mean` and `std_dev`.
///
/// Returns `None` when `std_dev` is zero.
#[must_use]
pub fn z_score(current: Decimal, mean: Decimal, std_dev: Decimal) -> Option<Decimal> {
    if std_dev == Decimal::ZERO {
        None
    } else {
        Some((current - mean) / std_dev)
    }
}

/// Clamps a decimal value into the inclusive `0..=100` range.
#[must_use]
pub fn clamp_0_100(value: Decimal) -> Decimal {
    if value < Decimal::ZERO {
        Decimal::ZERO
    } else {
        value.min(dec!(100))
    }
}

/// Returns the median of the provided `i64` values.
///
/// The input vector is sorted in place before computing the result.
///
/// For an odd number of values, the middle element is returned.
/// For an even number of values, the integer average of the two middle values
/// is returned.
///
/// Returns `None` when `values` is empty.
#[must_use]
pub fn median_i64(mut values: Vec<i64>) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some(i64::midpoint(values[mid - 1], values[mid]))
    }
}

/// Returns the lowest exchange-rate value in the provided slice.
///
/// The function compares only the numeric rate values and ignores timestamps.
///
/// Returns `None` when `values` is empty.
#[must_use]
pub fn lowest_value(values: &[ExchangeRate]) -> Option<&Decimal> {
    values.iter().map(ExchangeRate::rate).min()
}

/// Returns the highest exchange-rate value in the provided slice.
///
/// The function compares only the numeric rate values and ignores timestamps.
///
/// Returns `None` when `values` is empty.
#[must_use]
pub fn highest_value(values: &[ExchangeRate]) -> Option<&Decimal> {
    values.iter().map(ExchangeRate::rate).max()
}

#[test]
fn test_average() {
    let values = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let result = average(&values);
    assert_eq!(result, Some(dec!(3)));
}

#[test]
fn test_average_empty() {
    let values = vec![];
    let result = average(&values);
    assert_eq!(result, None);
}

#[test]
fn test_rolling_average_works() {
    let values = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let window_size = 3;
    let result = rolling_average(&values, window_size);
    assert_eq!(
        result,
        vec![None, None, Some(dec!(2)), Some(dec!(3)), Some(dec!(4))]
    );
}

#[test]
fn rolling_average_empty() {
    let values = vec![];
    let window_size = 3;
    let result = rolling_average(&values, window_size);
    assert_eq!(result, vec![None; values.len()]);
}

#[test]
fn rolling_average_invalid_window() {
    let values = vec![dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
    let window_size = 0;
    let result = rolling_average(&values, window_size);
    assert_eq!(result, vec![None; values.len()]);
}

#[test]
fn standard_deviation_works() {
    let values = vec![dec!(2), dec!(0), dec!(4), dec!(2)];
    let result = standard_deviation(&values);
    assert_eq!(result, dec!(2).sqrt());
}

#[test]
fn standard_deviation_empty() {
    let values = vec![];
    let result = standard_deviation(&values);
    assert_eq!(result, None);
}

#[test]
fn range_position_works() {
    let current = dec!(5);
    let high = dec!(10);
    let low = Decimal::ZERO;
    let result = range_position(current, high, low);
    assert_eq!(result, Some(dec!(0.5)));
}

#[test]
fn range_position_same_high_and_low() {
    let current = dec!(5);
    let high = dec!(5);
    let low = dec!(5);
    let result = range_position(current, high, low);
    assert_eq!(result, None);
}

#[test]
fn z_score_works() {
    let current = dec!(5);
    let mean = dec!(3);
    let std_dev = dec!(2);
    let result = z_score(current, mean, std_dev);
    assert_eq!(result, Some(dec!(1)));
}

#[test]
fn z_score_std_dev_zero() {
    let current = dec!(5);
    let mean = dec!(5);
    let std_dev = dec!(0);
    let result = z_score(current, mean, std_dev);
    assert_eq!(result, None);
}

#[test]
fn clamp_0_100_works() {
    assert_eq!(clamp_0_100(dec!(-10)), Decimal::ZERO);
    assert_eq!(clamp_0_100(dec!(50)), dec!(50));
    assert_eq!(clamp_0_100(dec!(150)), dec!(100));
}

#[test]
fn median_i64_works() {
    let values = vec![3, 1, 4, 1, 5];
    let result = median_i64(values);
    assert_eq!(result, Some(3));
    assert_eq!(median_i64(vec![]), None);
}

#[test]
fn test_lowest_value_non_empty() {
    let time = chrono::Utc::now();
    let values = vec![
        ExchangeRate::new(time, dec!(5)),
        ExchangeRate::new(time, dec!(2)),
        ExchangeRate::new(time, dec!(8)),
    ];
    let result = lowest_value(&values);
    assert_eq!(result, Some(&dec!(2)));
}
#[test]
fn test_lowest_value_empty() {
    let values: Vec<ExchangeRate> = vec![];
    let result = lowest_value(&values);
    assert_eq!(result, None);
}
#[test]
fn test_lowest_value_all_equal() {
    let time = chrono::Utc::now();
    let values = vec![
        ExchangeRate::new(time, dec!(3)),
        ExchangeRate::new(time, dec!(3)),
        ExchangeRate::new(time, dec!(3)),
    ];
    let result = lowest_value(&values);
    assert_eq!(result, Some(&dec!(3)));
}
#[test]
fn test_highest_value_non_empty() {
    let time = chrono::Utc::now();
    let values = vec![
        ExchangeRate::new(time, dec!(5)),
        ExchangeRate::new(time, dec!(2)),
        ExchangeRate::new(time, dec!(8)),
    ];
    let result = highest_value(&values);
    assert_eq!(result, Some(&dec!(8)));
}
#[test]
fn test_highest_value_empty() {
    let values: Vec<ExchangeRate> = vec![];
    let result = highest_value(&values);
    assert_eq!(result, None);
}
#[test]
fn test_highest_value_all_equal() {
    let time = chrono::Utc::now();
    let values = vec![
        ExchangeRate::new(time, dec!(3)),
        ExchangeRate::new(time, dec!(3)),
        ExchangeRate::new(time, dec!(3)),
    ];
    let result = highest_value(&values);
    assert_eq!(result, Some(&dec!(3)));
}
