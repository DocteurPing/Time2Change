/// Currency information (symbol and name).
#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    code: String,
    name: String,
}

impl CurrencyInfo {
    /// Create a new `CurrencyInfo`.
    #[must_use]
    pub const fn new(code: String, name: String) -> Self {
        Self { code, name }
    }

    /// Get the currency code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the currency name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
