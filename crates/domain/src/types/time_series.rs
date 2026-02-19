use crate::types::{currency_pair::CurrencyPair, exchange_rate::ExchangeRate};

pub struct TimeSeries {
    pair: CurrencyPair,
    rates: Vec<ExchangeRate>,
}

impl TimeSeries {
    pub fn new(pair: CurrencyPair, rates: Vec<ExchangeRate>) -> Self {
        Self { pair, rates }
    }

    pub fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    pub fn rates(&self) -> &[ExchangeRate] {
        &self.rates
    }

    pub fn add_rate(&mut self, rate: ExchangeRate) {
        self.rates.push(rate);
    }
}

impl std::fmt::Display for TimeSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeSeries({}, [", self.pair)?;
        for (i, rate) in self.rates.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{rate}")?;
        }
        write!(f, "])")
    }
}

#[test]
fn test_time_series_display() {
    let pair = CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap();
    let pair_string = pair.to_string();
    let time = chrono::Utc::now();
    let rates = vec![
        ExchangeRate::new(time, rust_decimal::dec!(1.2345)),
        ExchangeRate::new(time, rust_decimal::dec!(1.2346)),
    ];
    let time_series = TimeSeries::new(pair, rates.clone());
    assert_eq!(time_series.pair().to_string(), pair_string);
    assert_eq!(time_series.rates(), &rates);
    assert_eq!(
        format!("{time_series}"),
        format!("TimeSeries({pair_string}, [{time}: 1.2345, {time}: 1.2346])")
    );
}

#[test]
fn test_time_series_add_rate() {
    let pair = CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap();
    let time = chrono::Utc::now();
    let rate = ExchangeRate::new(time, rust_decimal::dec!(1.2345));
    let mut time_series = TimeSeries::new(pair, vec![]);
    time_series.add_rate(rate.clone());
    assert_eq!(time_series.rates().len(), 1);
    assert_eq!(time_series.rates(), &[rate]);
}
