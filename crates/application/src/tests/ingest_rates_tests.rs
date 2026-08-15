use std::collections::HashSet;

use chrono::Utc;
use domain::types::currency::Currency;
use rust_decimal::dec;

use crate::ports::rate_provider::RateProviderError;
use crate::ports::repository_errors::RepositoryError;
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

    let mut set = HashSet::new();
    set.insert(pair.quote().clone());
    let result = uc
        .fetch_rates_for_range(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap();
    assert_eq!(result, 1);
}

// ── range_already_ingested ──────────────────────────────────────

#[tokio::test]
async fn range_already_ingested_false_when_storage_is_empty() {
    let now = Utc::now();
    let pair = make_pair();

    let uc = IngestRatesUseCase::new(MockRepository::empty(), MockProvider::unused());
    let set: HashSet<Currency> = [pair.base().clone(), pair.quote().clone()].into();

    let already = uc
        .range_already_ingested(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap();

    assert!(!already);
}

#[tokio::test]
async fn range_already_ingested_true_when_every_pair_is_stored() {
    let now = Utc::now();
    let pair = make_pair();

    let repo = MockRepository::with_rates(pair.clone(), vec![make_rate(now, dec!(1.0850))]);
    let uc = IngestRatesUseCase::new(repo, MockProvider::unused());
    // Includes the base itself, which yields no pair and must be ignored.
    let set: HashSet<Currency> = [pair.base().clone(), pair.quote().clone()].into();

    let already = uc
        .range_already_ingested(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap();

    assert!(already);
}

#[tokio::test]
async fn range_already_ingested_false_when_one_pair_is_missing() {
    let now = Utc::now();
    let pair = make_pair();

    // EUR-USD is stored, EUR-GBP is not.
    let repo = MockRepository::with_rates(pair.clone(), vec![make_rate(now, dec!(1.0850))]);
    let uc = IngestRatesUseCase::new(repo, MockProvider::unused());
    let set: HashSet<Currency> = [
        pair.base().clone(),
        pair.quote().clone(),
        Currency::new("GBP").unwrap(),
    ]
    .into();

    let already = uc
        .range_already_ingested(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap();

    assert!(!already);
}

#[tokio::test]
async fn range_already_ingested_false_when_no_pair_can_be_built() {
    let now = Utc::now();
    let pair = make_pair();

    let uc = IngestRatesUseCase::new(MockRepository::empty(), MockProvider::unused());
    // Only the base currency: no pair to check, so nothing can be skipped.
    let set: HashSet<Currency> = [pair.base().clone()].into();

    let already = uc
        .range_already_ingested(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap();

    assert!(!already);
}

#[tokio::test]
async fn range_already_ingested_propagates_repository_errors() {
    let now = Utc::now();
    let pair = make_pair();

    let repo = MockRepository::with_error(RepositoryError::Storage("connection lost".into()));
    let uc = IngestRatesUseCase::new(repo, MockProvider::unused());
    let set: HashSet<Currency> = [pair.base().clone(), pair.quote().clone()].into();

    let error = uc
        .range_already_ingested(&set, now.date_naive(), now.date_naive(), pair.base())
        .await
        .unwrap_err();

    assert!(matches!(error, IngestError::Repository(_)));
}

#[tokio::test]
async fn range_already_ingested_ignores_rates_outside_the_range() {
    let now = Utc::now();
    let pair = make_pair();

    let repo = MockRepository::with_rates(pair.clone(), vec![make_rate(now, dec!(1.0850))]);
    let uc = IngestRatesUseCase::new(repo, MockProvider::unused());
    let set: HashSet<Currency> = [pair.base().clone(), pair.quote().clone()].into();

    let last_year = (now - chrono::Duration::days(365)).date_naive();
    let already = uc
        .range_already_ingested(&set, last_year, last_year, pair.base())
        .await
        .unwrap();

    assert!(!already);
}
