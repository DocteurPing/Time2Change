use chrono::Utc;
use rust_decimal::dec;

use crate::ports::exchange_rate_repository::RepositoryError;
use crate::ports::rate_provider::RateProviderError;
use crate::tests::helpers::{make_pair, make_rate};
use crate::tests::mocks::mock_provider::MockProvider;
use crate::tests::mocks::mock_repository::MockRepository;
use crate::use_cases::ingest_rates::{IngestError, IngestRatesUseCase};

#[tokio::test]
async fn ingest_error_display_provider() {
    let err = IngestError::Provider(RateProviderError::Timeout);
    assert_eq!(err.to_string(), "provider error: network timeout");
}

#[tokio::test]
async fn ingest_error_display_repository() {
    let err = IngestError::Repository(RepositoryError::Storage("connection lost".into()));
    assert_eq!(
        err.to_string(),
        "repository error: storage failure: connection lost"
    );
}

#[tokio::test]
async fn fetch_pair_range() {
    let now = Utc::now();
    let pair = make_pair();
    let rate = make_rate(now, dec!(1.0034));

    let provider = MockProvider::ok(rate.clone());
    let repo = MockRepository::empty();
    let uc = IngestRatesUseCase::new(repo, provider);

    let result = uc
        .fetch_rates_for_range(&pair, now.date_naive(), now.date_naive())
        .await
        .unwrap();
    assert_eq!(result, 1);
}
