use crate::types::currency_info::CurrencyInfo;
use crate::types::currency_pair::CurrencyPair;

/// Converts a slice of [`CurrencyInfo`] into a [`Vec`] of [`CurrencyPair`].
pub fn currency_info_list_to_currency_pairs(list_currency: &[CurrencyInfo]) -> Vec<CurrencyPair> {
    list_currency
        .iter()
        .flat_map(|currency| {
            list_currency
                .iter()
                .map(move |quote| CurrencyPair::new(currency.code().clone(), quote.code().clone()))
        })
        .filter_map(Result::ok)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::currency::Currency;

    #[test]
    fn test_currency_info_list_to_currency_pairs() {
        let currency_info_list = vec![
            CurrencyInfo::new(
                Currency::new("USD").unwrap(),
                "United States Dollar".to_owned(),
            ),
            CurrencyInfo::new(Currency::new("EUR").unwrap(), "Euro".to_owned()),
        ];
        let currency_pairs = currency_info_list_to_currency_pairs(&currency_info_list);
        assert_eq!(currency_pairs.len(), 2);
        assert!(
            currency_pairs.contains(
                &CurrencyPair::new(Currency::new("USD").unwrap(), Currency::new("EUR").unwrap())
                    .unwrap()
            )
        );
        assert!(
            currency_pairs.contains(
                &CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap())
                    .unwrap()
            )
        );
    }
}
