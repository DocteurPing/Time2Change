#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyError {
    Empty,
    InvalidLength,
    InvalidFormat,
}

pub(crate) struct Currency(String);

impl Currency {
    pub(crate) fn new(value: &str) -> Result<Currency, CurrencyError> {
        if value.is_empty() {
            return Err(CurrencyError::Empty);
        }
        if value.len() != 3 {
            return Err(CurrencyError::InvalidLength);
        }
        if !value.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CurrencyError::InvalidFormat);
        }
        Ok(Currency(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_currency() {
        let currency = Currency::new("USD");
        assert!(currency.is_ok());
    }

    #[test]
    fn empty_currency_fails() {
        let currency = Currency::new("");
        assert_eq!(currency.err(), Some(CurrencyError::Empty));
    }

    #[test]
    fn invalid_length_fails() {
        let currency = Currency::new("US");
        assert_eq!(currency.err(), Some(CurrencyError::InvalidLength));
    }

    #[test]
    fn lowercase_fails() {
        let currency = Currency::new("usd");
        assert_eq!(currency.err(), Some(CurrencyError::InvalidFormat));
    }
}
