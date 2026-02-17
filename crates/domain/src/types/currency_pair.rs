use crate::types::currency::Currency;

#[derive(Debug, PartialEq, Eq)]
pub enum CurrencyPairError {
    CurrencySame,
}

#[derive(Debug, PartialEq)]
pub struct CurrencyPair {
    base: Currency,
    quote: Currency,
}

impl CurrencyPair {
    pub fn new(base: Currency, quote: Currency) -> Result<Self, CurrencyPairError> {
        if base == quote {
            Err(CurrencyPairError::CurrencySame)
        } else {
            Ok(Self { base, quote })
        }
    }

    pub fn base(&self) -> &Currency {
        &self.base
    }

    pub fn quote(&self) -> &Currency {
        &self.quote
    }
}

impl std::fmt::Display for CurrencyPair {
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
    let pair = CurrencyPair::new(base, quote);

    assert!(pair.is_err());
    assert_eq!(pair.err().unwrap(), CurrencyPairError::CurrencySame);
}
