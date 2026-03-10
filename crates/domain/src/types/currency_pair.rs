use thiserror::Error;

use crate::types::currency::Currency;

/// Errors returned when constructing a [`CurrencyPair`].
#[derive(Debug, PartialEq, Eq, Error)]
pub enum CurrencyPairError {
    /// Indicates that the base and quote currencies are identical.
    ///
    /// A currency pair must consist of two distinct currencies.
    #[error("Base and quote currencies cannot be the same")]
    CurrencySame,
}

/// Represents an ordered foreign-exchange pair.
///
/// The `base` currency is the currency being priced, and the `quote` currency
/// is the currency used to express that price. For example, `EUR-USD` means
/// one euro priced in U.S. dollars.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CurrencyPair {
    base: Currency,
    quote: Currency,
}

impl CurrencyPair {
    /// Creates a new currency pair from a base and quote currency.
    ///
    /// # Errors
    ///
    /// Returns [`CurrencyPairError::CurrencySame`] when `base` and `quote`
    /// are the same currency.
    pub fn new(base: Currency, quote: Currency) -> Result<Self, CurrencyPairError> {
        if base == quote {
            Err(CurrencyPairError::CurrencySame)
        } else {
            Ok(Self { base, quote })
        }
    }

    /// Returns the base currency of the pair.
    #[must_use]
    pub const fn base(&self) -> &Currency {
        &self.base
    }

    /// Returns the quote currency of the pair.
    #[must_use]
    pub const fn quote(&self) -> &Currency {
        &self.quote
    }
}

impl std::fmt::Display for CurrencyPair {
    /// Formats the pair as `BASE-QUOTE`, such as `EUR-USD`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.base, self.quote)
    }
}

#[test]
fn test_currency_pair() {
    let base = Currency::new("EUR").unwrap();
    let quote = Currency::new("USD").unwrap();
    let pair = CurrencyPair::new(base, quote).unwrap();

    assert_eq!(pair.base().to_string(), "EUR");
    assert_eq!(pair.quote().to_string(), "USD");
}

#[test]
fn test_currency_same() {
    let base = Currency::new("EUR").unwrap();
    let quote = Currency::new("EUR").unwrap();
    let pair = CurrencyPair::new(base, quote).err().unwrap();

    assert_eq!(pair, CurrencyPairError::CurrencySame);
    assert_eq!(
        format!("{pair}"),
        "Base and quote currencies cannot be the same"
    );
}
