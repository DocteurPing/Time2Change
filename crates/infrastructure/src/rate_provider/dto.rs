use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Represents the response from the Frankfurter API for a date-range exchange rate request.
#[derive(Debug, Deserialize)]
pub struct FrankfurterRangeResponse {
    date: NaiveDate,
    base: String,
    quote: String,
    rate: Decimal,
}

impl FrankfurterRangeResponse {
    /// Returns the date of the exchange rates, as provided by the API.
    #[must_use]
    pub const fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// Returns the base currency code, as provided by the API.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Returns the quote currency code, as provided by the API.
    #[must_use]
    pub fn quote(&self) -> &str {
        &self.quote
    }

    /// Returns the exchange rate, as provided by the API.
    #[must_use]
    pub const fn rate(&self) -> Decimal {
        self.rate
    }
}

/// Represents a currency returned by the Frankfurter API.
#[derive(Debug, Deserialize)]
pub struct FrankfurterCurrenciesResponse {
    iso_code: String,
    name: String,
}

impl FrankfurterCurrenciesResponse {
    /// Returns the ISO currency code, as provided by the API.
    #[must_use]
    pub fn iso_code(&self) -> &str {
        &self.iso_code
    }

    /// Returns the currency name, as provided by the API.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
