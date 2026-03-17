use chrono::NaiveDate;
use serde::Deserialize;

/// Represents the response from the Frankfurter API for exchange rates.
#[derive(Debug, Deserialize)]
pub struct FrankfurterRateProviderResponse {
    #[allow(dead_code)]
    // the API response includes the base currency, but we don't need it
    base: String,
    date: NaiveDate,
    rates: std::collections::HashMap<String, f64>,
}

impl FrankfurterRateProviderResponse {
    /// Returns the date of the exchange rates, as provided by the API.
    #[must_use]
    pub const fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// Returns the exchange rates map, where keys are quote currency codes and values are the corresponding exchange rates.
    #[must_use]
    pub const fn rates(&self) -> &std::collections::HashMap<String, f64> {
        &self.rates
    }
}
