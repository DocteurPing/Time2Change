use std::collections::HashMap;

use application::ports::exchange_rate_repository::ExchangeRateRepository;
use chrono::{DateTime, TimeZone, Utc};
use domain::types::currency::Currency;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use rust_decimal::dec;
use sqlx::PgPool;

use crate::exchange_rate::repository::PostgresExchangeRateRepository;

fn make_pair() -> CurrencyPair {
    CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap()
}

fn truncate_to_micros(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    let seconds = timestamp.timestamp();
    let nanos = timestamp.timestamp_subsec_micros() * 1_000;
    Utc.timestamp_opt(seconds, nanos)
        .single()
        .unwrap_or(timestamp)
}

// sqlx::test automatically spins up a temporary database,
// runs migrations, and tears it down after the test.
#[sqlx::test]
async fn save_and_load_round_trip(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = truncate_to_micros(Utc::now());
    let pair = make_pair();
    let rate = ExchangeRate::new(now, dec!(1.0850));

    let mut map = HashMap::new();
    map.insert(pair.clone(), vec![rate]);
    repo.save_rates(map).await.unwrap();

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates().get(&now), Some(&dec!(1.0850)));

    assert!(repo.exists(&pair, &range).await.unwrap());
}

#[sqlx::test]
async fn exists_no_data(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);
    let now = truncate_to_micros(Utc::now());
    assert!(!repo.exists(&make_pair(), &(now..=now)).await.unwrap());
}

#[sqlx::test]
async fn save_multiple_rates(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = truncate_to_micros(Utc::now());
    let pair = make_pair();
    let rates = vec![
        ExchangeRate::new(now, dec!(1.0850)),
        ExchangeRate::new(now + chrono::Duration::seconds(60), dec!(1.0860)),
        ExchangeRate::new(now + chrono::Duration::seconds(120), dec!(1.0870)),
    ];

    let mut map = HashMap::new();
    map.insert(pair.clone(), rates);
    repo.save_rates(map).await.unwrap();

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(120));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 3);
    assert_eq!(series.rates().get(&now), Some(&dec!(1.0850)));
    assert_eq!(
        series.rates().get(&(now + chrono::Duration::seconds(60))),
        Some(&dec!(1.0860))
    );
    assert_eq!(
        series.rates().get(&(now + chrono::Duration::seconds(120))),
        Some(&dec!(1.0870))
    );
}

#[sqlx::test]
async fn save_duplicate_rates(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = truncate_to_micros(Utc::now());
    let pair = make_pair();
    let rate1 = ExchangeRate::new(now, dec!(1.0850));
    let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

    let mut map1 = HashMap::new();
    map1.insert(pair.clone(), vec![rate1]);
    repo.save_rates(map1).await.unwrap();

    let mut map2 = HashMap::new();
    map2.insert(pair.clone(), vec![rate2]);
    repo.save_rates(map2).await.unwrap(); // should be ignored

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates().get(&now), Some(&dec!(1.0850))); // original rate should be preserved
}

#[sqlx::test]
async fn save_duplicate_in_batch(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = truncate_to_micros(Utc::now());
    let pair = make_pair();
    let rate1 = ExchangeRate::new(now, dec!(1.0850));
    let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

    let mut map = HashMap::new();
    map.insert(pair.clone(), vec![rate1, rate2]);
    repo.save_rates(map).await.unwrap(); // one of them should be ignored

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates().get(&now), Some(&dec!(1.0850))); // original rate should be preserved
}

#[sqlx::test]
async fn test_migration(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);
    let result = repo.migrate().await;
    assert!(result.is_ok());
}
