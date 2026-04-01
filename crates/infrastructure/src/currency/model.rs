use application::ports::repository_errors::RepositoryError;
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;

/// Raw database row returned by the `currencies` table.
///
/// Both columns are decoded as plain `String` values by sqlx — validation
/// into domain types happens at the infrastructure boundary via [`TryFrom`],
/// keeping the `domain` crate free of any sqlx dependency.
///
/// The field types map directly to the Postgres column types:
/// - `currency` is `TEXT` — sqlx decodes it as `String` natively
/// - `name` is `TEXT` — sqlx decodes it as `String` natively
#[derive(Debug, sqlx::FromRow)]
pub struct CurrencyInfoRow {
    pub(crate) currency: String,
    pub(crate) name: String,
}

impl TryFrom<CurrencyInfoRow> for CurrencyInfo {
    type Error = RepositoryError;

    /// Converts a raw DB row into the domain [`CurrencyInfo`] type.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Invalid`] if the stored currency code
    /// fails [`Currency`] validation (not a 3-letter uppercase ASCII string).
    /// This should never happen with well-formed data but is handled
    /// explicitly so that corrupt rows surface as a clear error rather than
    /// a panic.
    fn try_from(row: CurrencyInfoRow) -> Result<Self, Self::Error> {
        let code =
            Currency::new(&row.currency).map_err(|e| RepositoryError::Invalid(e.to_string()))?;
        Ok(Self::new(code, row.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_info_row_converts_to_domain_type() {
        let row = CurrencyInfoRow {
            currency: "EUR".to_owned(),
            name: "Euro".to_owned(),
        };
        let info = CurrencyInfo::try_from(row).unwrap();
        assert_eq!(info.code().to_string(), "EUR");
        assert_eq!(info.name(), "Euro");
    }

    #[test]
    fn currency_info_row_invalid_code_returns_error() {
        let row = CurrencyInfoRow {
            currency: "invalid".to_owned(),
            name: "Bad Currency".to_owned(),
        };
        let result = CurrencyInfo::try_from(row);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::Invalid(_)));
    }
}
