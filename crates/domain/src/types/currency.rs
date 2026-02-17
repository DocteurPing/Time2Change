#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyError {
    InvalidFormat,
    InvalidLength,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Currency([u8; 3]);

impl Currency {
    pub fn new(value: &str) -> Result<Self, CurrencyError> {
        if value.len() != 3 {
            return Err(CurrencyError::InvalidLength);
        }
        if !value.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CurrencyError::InvalidFormat);
        }
        let bytes = value
            .as_bytes()
            .try_into()
            .map_err(|_| CurrencyError::InvalidLength)?;
        Ok(Self(bytes))
    }
}

impl TryFrom<&str> for Currency {
    type Error = CurrencyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Currency::new(value)
    }
}

impl std::fmt::Display for CurrencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "Currency must be 3 letters"),
            Self::InvalidFormat => write!(f, "Currency must be uppercase ASCII letters"),
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let currency = std::str::from_utf8(&self.0).expect("Currency invariant violated");
        write!(f, "{currency}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_currency() {
        let currency = Currency::new("USD");
        assert!(currency.is_ok());
        assert_eq!(currency.unwrap().to_string(), "USD");
    }

    #[test]
    fn empty_currency_fails() {
        let currency = Currency::new("");
        let error = currency.err().unwrap();
        assert_eq!(error, CurrencyError::InvalidLength);
        assert_eq!(error.to_string(), "Currency must be 3 letters")
    }

    #[test]
    fn invalid_length_fails() {
        let currency = Currency::new("US");
        let error = currency.err().unwrap();
        assert_eq!(error, CurrencyError::InvalidLength);
        assert_eq!(error.to_string(), "Currency must be 3 letters")
    }

    #[test]
    fn lowercase_fails() {
        let currency = Currency::new("usd");
        let error = currency.err().unwrap();
        assert_eq!(error, CurrencyError::InvalidFormat);
        assert_eq!(
            error.to_string(),
            "Currency must be uppercase ASCII letters"
        )
    }

    #[test]
    fn non_letter_fails() {
        let currency = Currency::new("0UR");
        let error = currency.err().unwrap();
        assert_eq!(error, CurrencyError::InvalidFormat);
        assert_eq!(
            error.to_string(),
            "Currency must be uppercase ASCII letters"
        )
    }
}
