use rust_decimal::{Decimal, MathematicalOps};

pub fn average(values: &[Decimal]) -> Option<Decimal> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<Decimal>() / Decimal::from(values.len()))
    }
}

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

pub fn volatility(values: &[Decimal], window: usize) -> Vec<Option<Decimal>> {
    let len = values.len();

    if window == 0 || window > len {
        return vec![None; len];
    }

    let mut result = vec![None; window - 1];

    let mut sum = Decimal::ZERO;
    let mut sum_sq = Decimal::ZERO;

    // Initialize first window
    for &value in &values[..window] {
        sum += value;
        sum_sq += value * value;
    }

    let mean = sum / Decimal::from(window);
    let variance = (sum_sq / Decimal::from(window)) - (mean * mean);
    result.push(variance.sqrt());

    // Slide window
    for i in window..len {
        let incoming = values[i];
        let outgoing = values[i - window];

        sum += incoming - outgoing;
        sum_sq += incoming * incoming - outgoing * outgoing;

        let mean = sum / Decimal::from(window);
        let variance = (sum_sq / Decimal::from(window)) - (mean * mean);

        result.push(variance.sqrt());
    }

    result
}

pub fn range_position(current: Decimal, high: Decimal, low: Decimal) -> Option<Decimal> {
    if high == low {
        None
    } else {
        Some((current - low) / (high - low))
    }
}

pub fn z_score(current: Decimal, mean: Decimal, std_dev: Decimal) -> Option<Decimal> {
    if std_dev == Decimal::ZERO {
        None
    } else {
        Some((current - mean) / std_dev)
    }
}

#[test]
fn test_average() {
    let values = vec![
        Decimal::from(1),
        Decimal::from(2),
        Decimal::from(3),
        Decimal::from(4),
        Decimal::from(5),
    ];
    let result = average(&values);
    assert_eq!(result, Some(Decimal::from(3)));
}

#[test]
fn test_average_empty() {
    let values = vec![];
    let result = average(&values);
    assert_eq!(result, None);
}

#[test]
fn rolling_average_works() {
    let values = vec![
        Decimal::from(1),
        Decimal::from(2),
        Decimal::from(3),
        Decimal::from(4),
        Decimal::from(5),
    ];
    let window_size = 3;
    let result = rolling_average(&values, window_size);
    assert_eq!(
        result,
        vec![
            None,
            None,
            Some(Decimal::from(2)),
            Some(Decimal::from(3)),
            Some(Decimal::from(4))
        ]
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
    let values = vec![
        Decimal::from(1),
        Decimal::from(2),
        Decimal::from(3),
        Decimal::from(4),
        Decimal::from(5),
    ];
    let window_size = 0;
    let result = rolling_average(&values, window_size);
    assert_eq!(result, vec![None; values.len()]);
}

#[test]
fn standard_deviation_works() {
    let values = vec![
        Decimal::from(2),
        Decimal::from(0),
        Decimal::from(4),
        Decimal::from(2),
    ];
    let result = standard_deviation(&values);
    assert_eq!(result, Decimal::from(2).sqrt());
}

#[test]
fn standard_deviation_empty() {
    let values = vec![];
    let result = standard_deviation(&values);
    assert_eq!(result, None);
}

#[test]
fn volatility_works() {
    let values = vec![
        Decimal::from(2),
        Decimal::from(0),
        Decimal::from(4),
        Decimal::from(2),
    ];
    let window_size = 2;
    let result = volatility(&values, window_size);
    dbg!(&result);
    assert_eq!(
        result,
        vec![
            None,
            Some(Decimal::from(1)),
            Some(Decimal::from(2)),
            Some(Decimal::from(1))
        ]
    );
}

#[test]
fn volatility_empty() {
    let values = vec![];
    let window_size = 2;
    let result = volatility(&values, window_size);
    assert_eq!(result, vec![None; values.len()]);
}

#[test]
fn volatility_invalid_window() {
    let values = vec![
        Decimal::from(1),
        Decimal::from(2),
        Decimal::from(3),
        Decimal::from(4),
        Decimal::from(5),
    ];
    let window_size = 0;
    let result = volatility(&values, window_size);
    assert_eq!(result, vec![None; values.len()]);
}

#[test]
fn range_position_works() {
    let current = Decimal::from(5);
    let high = Decimal::from(10);
    let low = Decimal::ZERO;
    let result = range_position(current, high, low);
    assert_eq!(result, Decimal::from_f32_retain(0.5));
}

#[test]
fn range_position_same_high_and_low() {
    let current = Decimal::from(5);
    let high = Decimal::from(5);
    let low = Decimal::from(5);
    let result = range_position(current, high, low);
    assert_eq!(result, None);
}

#[test]
fn z_score_works() {
    let current = Decimal::from(5);
    let mean = Decimal::from(3);
    let std_dev = Decimal::from(2);
    let result = z_score(current, mean, std_dev);
    assert_eq!(result, Some(Decimal::from(1)));
}

#[test]
fn z_score_std_dev_zero() {
    let current = Decimal::from(5);
    let mean = Decimal::from(5);
    let std_dev = Decimal::from(0);
    let result = z_score(current, mean, std_dev);
    assert_eq!(result, None);
}
