use std::collections::HashSet;

use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;

use crate::ports::rate_provider::RateProviderError;
use crate::ports::repository_errors::RepositoryError;
use crate::tests::mocks::mock_provider::MockProvider;
use crate::tests::mocks::mock_repository::MockRepository;
use crate::use_cases::sync_currencies::{SyncCurrenciesError, SyncCurrenciesUseCase};

fn make_currency(code: &str, name: &str) -> CurrencyInfo {
    CurrencyInfo::new(Currency::new(code).unwrap(), name.to_owned())
}

#[tokio::test]
async fn sync_error_display_provider() {
    let err = SyncCurrenciesError::Provider(RateProviderError::Timeout);
    assert_eq!(err.to_string(), "provider error: network timeout");
}

#[tokio::test]
async fn sync_error_display_repository() {
    let err = SyncCurrenciesError::Repository(RepositoryError::Storage("connection lost".into()));
    assert_eq!(
        err.to_string(),
        "repository error: storage failure: connection lost"
    );
}

#[tokio::test]
async fn execute_persists_all_fetched_items() {
    let fetched = vec![
        make_currency("EUR", "Euro"),
        make_currency("USD", "US Dollar"),
        make_currency("JPY", "Japanese Yen"),
    ];

    let provider = MockProvider::with_currencies_ok(fetched.clone());
    let repo = MockRepository::empty();
    let uc = SyncCurrenciesUseCase::new(repo, provider, HashSet::new());

    let result = uc.execute().await.unwrap();
    assert_eq!(result, fetched.len());

    let persisted = uc.list_currencies().await.unwrap();
    assert_eq!(persisted, fetched);
}

#[tokio::test]
async fn execute_returns_provider_error() {
    let provider = MockProvider::with_currencies_err(RateProviderError::ApiError(
        "503 Service Unavailable".to_owned(),
    ));
    let repo = MockRepository::empty();
    let uc = SyncCurrenciesUseCase::new(repo, provider, HashSet::new());

    let err = uc.execute().await.unwrap_err();
    assert!(matches!(
        err,
        SyncCurrenciesError::Provider(RateProviderError::ApiError(_))
    ));
}

#[tokio::test]
async fn execute_returns_repository_error() {
    let fetched = vec![make_currency("EUR", "Euro")];

    let provider = MockProvider::with_currencies_ok(fetched);
    let repo = MockRepository::with_save_error(RepositoryError::Storage("write failed".into()));
    let uc = SyncCurrenciesUseCase::new(repo, provider, HashSet::new());

    let err = uc.execute().await.unwrap_err();
    assert!(matches!(
        err,
        SyncCurrenciesError::Repository(RepositoryError::Storage(_))
    ));
}

#[tokio::test]
async fn execute_with_currency_filter() {
    let fetched = vec![
        make_currency("EUR", "Euro"),
        make_currency("USD", "US Dollar"),
        make_currency("GBP", "British Pound"),
    ];

    let provider = MockProvider::with_currencies_ok(fetched.clone());
    let repo = MockRepository::empty();
    let selected_currencies = vec![Currency::new("EUR").unwrap(), Currency::new("GBP").unwrap()];
    let selected_currencies_size = selected_currencies.len();
    let uc = SyncCurrenciesUseCase::new(repo, provider, HashSet::from_iter(selected_currencies));

    let result = uc.execute().await.unwrap();
    assert_eq!(result, selected_currencies_size);

    let persisted = uc.list_currencies().await.unwrap();
    assert_eq!(
        persisted,
        vec![
            make_currency("EUR", "Euro"),
            make_currency("GBP", "British Pound")
        ]
    );
}
