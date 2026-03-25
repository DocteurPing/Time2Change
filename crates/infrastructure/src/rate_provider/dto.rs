use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Deserialize;

/// Represents the response from the Frankfurter API for a single-date exchange
/// rate request.
#[derive(Debug, Deserialize)]
pub struct FrankfurterRateProviderResponse {
    date: NaiveDate,
    rates: HashMap<String, f64>,
}

impl FrankfurterRateProviderResponse {
    /// Returns the date of the exchange rates, as provided by the API.
    #[must_use]
    pub const fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// Returns the exchange rates map, where keys are quote currency codes and
    /// values are the corresponding exchange rates.
    #[must_use]
    pub const fn rates(&self) -> &HashMap<String, f64> {
        &self.rates
    }
}

/// Represents the response from the Frankfurter API for a date-range exchange
/// rate request (e.g. `2000-01-01..2000-12-31`).
///
/// The `rates` map is keyed by date string (`"YYYY-MM-DD"`) and each value is
/// itself a map from quote currency code to the exchange rate on that date.
#[derive(Debug, Deserialize)]
pub struct FrankfurterRangeResponse {
    start_date: NaiveDate,
    end_date: NaiveDate,
    /// Outer key: date string `"YYYY-MM-DD"`.
    /// Inner key: quote currency code (e.g. `"USD"`).
    rates: HashMap<String, HashMap<String, f64>>,
}

impl FrankfurterRangeResponse {
    /// Returns the first date covered by this response.
    #[must_use]
    pub const fn start_date(&self) -> &NaiveDate {
        &self.start_date
    }

    /// Returns the last date covered by this response.
    #[must_use]
    pub const fn end_date(&self) -> &NaiveDate {
        &self.end_date
    }

    /// Returns the full nested rates map.
    ///
    /// The outer key is a date string (`"YYYY-MM-DD"`); the inner key is the
    /// quote currency code.
    #[must_use]
    pub const fn rates(&self) -> &HashMap<String, HashMap<String, f64>> {
        &self.rates
    }
}
