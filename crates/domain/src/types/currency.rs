use thiserror::Error;

/// Errors that can occur while creating a [`Currency`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CurrencyError {
    /// The provided value is not made of uppercase ASCII letters.
    #[error("Currency must be uppercase ASCII letters: {value}")]
    InvalidFormat {
        /// The invalid input value that failed validation.
        value: String,
    },

    /// The provided value does not contain exactly three characters.
    #[error("Currency must be 3 letters")]
    InvalidLength,
}

/// ISO-style three-letter uppercase currency code.
///
/// This value object stores the currency code in a compact fixed-size byte
/// array and guarantees that every instance contains exactly three uppercase
/// ASCII letters such as `EUR`, `USD`, or `JPY`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Currency {
    currency: [u8; 3],
}

impl Currency {
    /// Creates a new validated currency code.
    ///
    /// # Errors
    ///
    /// Returns [`CurrencyError::InvalidLength`] when `value` does not contain
    /// exactly three characters.
    ///
    /// Returns [`CurrencyError::InvalidFormat`] when `value` contains any
    /// character that is not an uppercase ASCII letter.
    pub fn new(value: &str) -> Result<Self, CurrencyError> {
        if value.len() != 3 {
            return Err(CurrencyError::InvalidLength);
        }
        if !value.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CurrencyError::InvalidFormat {
                value: value.to_owned(),
            });
        }
        let bytes = value
            .as_bytes()
            .try_into()
            .map_err(|_| CurrencyError::InvalidLength)?;
        Ok(Self { currency: bytes })
    }
}

impl TryFrom<&str> for Currency {
    type Error = CurrencyError;

    /// Attempts to build a [`Currency`] from a string slice.
    ///
    /// This is equivalent to calling [`Currency::new`].
    ///
    /// # Errors
    ///
    /// Propagates the same validation errors as [`Currency::new`].
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Display for Currency {
    /// Formats the currency as its three-letter code.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let currency = std::str::from_utf8(&self.currency).map_err(|_| std::fmt::Error)?;
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
        assert_eq!(error.to_string(), "Currency must be 3 letters");
    }

    #[test]
    fn invalid_length_fails() {
        let currency = Currency::new("US");
        let error = currency.err().unwrap();
        assert_eq!(error, CurrencyError::InvalidLength);
        assert_eq!(error.to_string(), "Currency must be 3 letters");
    }

    #[test]
    fn lowercase_fails() {
        let currency = Currency::new("usd");
        let error = currency.err().unwrap();
        assert_eq!(
            error,
            CurrencyError::InvalidFormat {
                value: "usd".to_owned()
            }
        );
        assert_eq!(
            error.to_string(),
            "Currency must be uppercase ASCII letters: usd"
        );
    }

    #[test]
    fn non_letter_fails() {
        let currency = Currency::new("0UR");
        let error = currency.err().unwrap();
        assert_eq!(
            error,
            CurrencyError::InvalidFormat {
                value: "0UR".to_owned()
            }
        );
        assert_eq!(
            error.to_string(),
            "Currency must be uppercase ASCII letters: 0UR"
        );
    }
}
