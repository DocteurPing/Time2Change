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
