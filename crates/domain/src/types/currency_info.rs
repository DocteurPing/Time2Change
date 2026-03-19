use crate::types::currency::Currency;

/// Currency information (symbol and name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyInfo {
    code: Currency,
    name: String,
}

impl CurrencyInfo {
    /// Create a new `CurrencyInfo`.
    #[must_use]
    pub const fn new(code: Currency, name: String) -> Self {
        Self { code, name }
    }

    /// Get the currency code.
    #[must_use]
    pub const fn code(&self) -> &Currency {
        &self.code
    }

    /// Get the currency name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
